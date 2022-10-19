use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::Receiver;
use std::thread;
#[cfg(not(test))]
use log::{info, warn, trace, debug}; // Use log crate when building application
#[cfg(test)]
use std::{println as info, println as warn, println as trace, println as debug};
use std::fmt::Debug;
use std::hash::Hash;
use spargebra::Query;
use crate::rsp::s2r::{ContentContainer, CSPARQLWindow, Report, ReportStrategy, Tick, WindowTriple};
use crate::{ Syntax, Triple, TripleStore};
use crate::rsp::r2s::{Relation2StreamOperator, StreamOperator};
use crate::rsp::r2r::{R2ROperator};
use crate::sparql::{Binding, eval_query, evaluate_plan_and_debug};

pub mod s2r;
pub mod r2r;
pub mod r2s;

pub enum OperationMode{
    SingleThread, MultiThread
}
pub struct  RSPBuilder<'a, I, O> {
    width: usize,
    slide: usize,
    tick: Option<Tick>,
    report_strategy: Option<ReportStrategy>,
    triples: Option<&'a str>,
    syntax: Option<Syntax>,
    rules: Option<&'a str>,
    query_str: Option<&'a str>,
    result_consumer: Option<ResultConsumer<O>>,
    r2s: Option<StreamOperator>,
    r2r: Option<Box<dyn R2ROperator<I, O>>>,
    operation_mode : OperationMode

}
impl <'a, I, O> RSPBuilder<'a, I, O> where O: Clone + Hash + Eq + Send + Debug +'static, I: Eq + PartialEq + Clone + Debug + Hash + Send +'static{
    pub fn new(width: usize, slide: usize)-> RSPBuilder<'a, I, O>{
        RSPBuilder{
            width,
            slide,
            tick: None,
            report_strategy: None,
            triples: None,
            syntax: None,
            rules: None,
            query_str: None,
            result_consumer: None,
            r2s: None,
            r2r: None,
            operation_mode: OperationMode::MultiThread
        }
    }
    pub fn add_tick(mut self, tick: Tick)->RSPBuilder<'a, I, O>{
        self.tick=Some(tick);
        self
    }
    pub fn add_report_strategy(mut self, strategy: ReportStrategy)->RSPBuilder<'a, I, O>{
        self.report_strategy= Some(strategy);
        self
    }
    pub fn add_triples(mut self, triples: &'a str)->RSPBuilder<'a, I, O>{
        self.triples= Some(triples);
        self
    }
    pub fn add_rules(mut self, rules: &'a str)->RSPBuilder<'a, I, O>{
        self.rules= Some(rules);
        self
    }
    pub fn add_query(mut self, query: &'a str)->RSPBuilder<'a, I, O>{
        self.query_str= Some(query);
        self
    }
    pub fn add_consumer(mut self, consumer: ResultConsumer<O>)->RSPBuilder<'a, I, O>{
        self.result_consumer= Some(consumer);
        self
    }
    pub fn add_r2s(mut self, r2s: StreamOperator)->RSPBuilder<'a, I, O>{
        self.r2s= Some(r2s);
        self
    }
    pub fn add_r2r(mut self, r2r: Box<dyn R2ROperator<I, O>>)->RSPBuilder<'a, I, O>{
        self.r2r= Some(r2r);
        self
    }
    pub fn add_syntax(mut self, syntax: Syntax)->RSPBuilder<'a, I, O>{
        self.syntax = Some(syntax);
        self
    }
    pub fn set_operation_mode(mut self, operation_mode: OperationMode)->RSPBuilder<'a, I, O>{
        self.operation_mode = operation_mode;
        self
    }
    pub fn build(self) -> RSPEngine<I,O>{
        RSPEngine::new(
            self.width,
            self.slide,
        self.tick.unwrap_or_default(),
        self.report_strategy.unwrap_or_default(),
        self.triples.unwrap_or(""),
        self.syntax.unwrap_or_default(),
        self.rules.unwrap_or(""),
        self.query_str.expect("Please provide R2R query"),
        self.result_consumer.unwrap_or(ResultConsumer{function: Arc::new( Box::new(|r|println!("Bindings: {:?}",r)))}),
            self.r2s.unwrap_or_default(),
            self.r2r.expect("Please provide R2R operator!"),
            self.operation_mode
        )

    }
}
pub struct RSPEngine<I,O> where I: Eq + PartialEq + Clone + Debug + Hash + Send{
    s2r: CSPARQLWindow<I>,
    r2r: Arc<Mutex<Box<dyn R2ROperator<I,O>>>>,
    r2s_consumer: ResultConsumer<O>,
    r2s_operator: Arc<Mutex<Relation2StreamOperator<O>>>
}
pub struct ResultConsumer<I>{
    pub function: Arc<dyn Fn(I) ->() + Send + Sync>
}


impl  <I, O> RSPEngine<I, O> where O: Clone + Hash + Eq + Send +'static, I: Eq + PartialEq + Clone + Debug + Hash + Send +'static{

    pub fn new(width: usize, slide: usize, tick: Tick, report_strategy: ReportStrategy, triples: &str, syntax: Syntax, rules: &str, query_str: &str, result_consumer: ResultConsumer<O>, r2s: StreamOperator, r2r: Box<dyn R2ROperator<I, O>>, operation_mode: OperationMode) -> RSPEngine<I, O>{
        let mut report = Report::new();
        report.add(report_strategy);
        let mut window = CSPARQLWindow::new(width, slide, report, tick);
        let mut store = r2r;

        store.load_triples(triples, syntax);
        store.load_rules(rules);
        let query = Query::parse(query_str, None).unwrap();
        let mut engine = RSPEngine{s2r: window, r2r:  Arc::new(Mutex::new(store)), r2s_consumer: result_consumer, r2s_operator: Arc::new(Mutex::new(Relation2StreamOperator::new(r2s,0)))};
        match operation_mode {
            OperationMode::SingleThread => {
                let consumer_temp = engine.r2r.clone();
                let r2s_consumer = engine.r2s_consumer.function.clone();
                let mut r2s_operator = engine.r2s_operator.clone();
                let call_back: Box<dyn FnMut(ContentContainer<I>) -> ()> = Box::new(move |content| {
                    Self::evaluate_r2r_and_call_r2s(&query, consumer_temp.clone(), r2s_consumer.clone(), r2s_operator.clone(), content);
                });
                engine.s2r.register_callback(call_back);
            },
            OperationMode::MultiThread => {
                let consumer = engine.s2r.register();
                engine.register_r2r(consumer, query);
            }
        }


        engine
    }
    fn register_r2r(&mut self,receiver: Receiver<ContentContainer<I>>, query: Query){
        let consumer_temp = self.r2r.clone();
        let r2s_consumer = self.r2s_consumer.function.clone();
        let mut r2s_operator = self.r2s_operator.clone();
        thread::spawn(move||{
            loop{
                match receiver.recv(){
                    Ok(mut content)=> {
                        Self::evaluate_r2r_and_call_r2s(&query, consumer_temp.clone(), r2s_consumer.clone(), r2s_operator.clone(), content);
                    },
                    Err(_) => {
                        debug!("Shutting down!");
                        break;
                    }
                }
            }
            debug!("Shutdown complete!");
        });
    }

    fn evaluate_r2r_and_call_r2s(query: &Query, consumer_temp: Arc<Mutex<Box<dyn R2ROperator<I, O>>>>, r2s_consumer: Arc<dyn Fn(O) + Send + Sync>, mut r2s_operator: Arc<Mutex<Relation2StreamOperator<O>>>, mut content: ContentContainer<I>) {
        debug!("R2R operator retrieved graph {:?}", content);
        let time_stamp = content.get_last_timestamp_changed();
        let mut store = consumer_temp.lock().unwrap();
        content.clone().into_iter().for_each(|t| {
            store.add(t);
        });
        let inferred = store.materialize();
        let r2r_result = store.execute_query(&query);
        let r2s_result = r2s_operator.lock().unwrap().eval(r2r_result, time_stamp);
        // TODO run R2S in other thread
        for result in r2s_result {
            (r2s_consumer)(result);
        }
        //remove data from stream
        content.iter().for_each(|t| {
            store.remove(t);
        });
        inferred.iter().for_each(|t|{
            store.remove(t);
        });
    }


    pub fn add(&mut self, event_item: I, ts: usize) {
        self.s2r.add_to_window(event_item,ts);
    }
    pub fn stop(&mut self){
        self.s2r.stop();
    }
}

pub struct SimpleR2R {
    pub item: TripleStore
}
impl R2ROperator<WindowTriple,Vec<Binding>> for SimpleR2R {
    fn load_triples(&mut self, data: &str, syntax: Syntax) -> Result<(), &'static str> {
        self.item.load_triples(data,syntax)
    }

    fn load_rules(&mut self, data: &str) -> Result<(), &'static str> {
        self.item.load_rules(data)
    }

    fn add(&mut self, data: WindowTriple) {
        let encoded_triple = Triple::from(data.s,data.p,data.o,&mut self.item.encoder);
        self.item.add(encoded_triple);
    }

    fn remove(&mut self, data: &WindowTriple) {
        let encoded_triple = Triple::from(data.s.clone(),data.p.clone(),data.o.clone(),&mut self.item.encoder);

        self.item.remove_ref(&encoded_triple);
    }

    fn materialize(&mut self) -> Vec<WindowTriple>{
        let inferred = self.item.materialize();
        inferred.into_iter().map(|t|WindowTriple{s:self.item.encoder.decode(&t.s.to_encoded()).unwrap().to_string(),
        p:self.item.encoder.decode(&t.p.to_encoded()).unwrap().to_string(),
        o:self.item.encoder.decode(&t.o.to_encoded()).unwrap().to_string()}).collect()
    }

    fn execute_query(&self, query: &Query) -> Vec<Vec<Binding>> {
        let mut encoder = self.item.encoder.clone();
        let plan = eval_query(&query, &self.item.triple_index, &mut encoder);
        let iterator = evaluate_plan_and_debug(&plan, &self.item.triple_index, &mut encoder);
        iterator.collect()
    }
}

#[cfg(test)]
mod tests{
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::io::{Write, BufRead};

    use std::time::Duration;
    use super::*;

    #[test]
    #[ignore]
    fn rsp_integration(){
        let ntriples_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let rules = "@prefix test: <http://www.w3.org/test/>.\n{?x <http://test.be/hasVal> ?y. ?y <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> test:SuperType.}";
        let function = Box::new(|r|println!("Bindings: {:?}",r));
        let result_consumer = ResultConsumer{function: Arc::new(function)};
        let r2r = Box::new(SimpleR2R {item: TripleStore::new()});
        let mut engine = RSPBuilder::new(10,2)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(ntriples_file)
            .add_syntax(Syntax::NTriples)
            .add_rules(rules)
            .add_query("Select * WHERE{ ?s a <http://www.w3.org/test/SuperType>}")
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .build();
        for i in 0..10 {
            let triple = WindowTriple{s:format!("s{}", i), p:"<http://test.be/hasVal>".to_string(),o: "<http://example.com/foo>".to_string()};

            engine.add(triple,i);
        }
        engine.stop();
        thread::sleep(Duration::from_secs(2));

    }
    #[test]
    #[ignore]
    fn test_load_from_file(){
        let ntriples_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let rules = "@prefix test: <http://www.w3.org/test/>.\n{?x <http://test.be/hasVal> ?y. ?y <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> test:SuperType.}";

        //write to file
        let function = Box::new(|r|{
            let mut output = OpenOptions::new()
                .write(true)
                .append(true)
                .open("/tmp/rox_rsp.out")
                .unwrap();
            write!(output, "Bindings: {:?}\n",r).unwrap();
                });
        let result_consumer = ResultConsumer{function: Arc::new(function)};
        let r2r = Box::new(SimpleR2R {item: TripleStore::new()});
        let mut engine = RSPBuilder::new(10,2)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(ntriples_file)
            .add_syntax(Syntax::NTriples)
            .add_rules(rules)
            .add_query("Select * WHERE{ ?s <http://test/hasLocation> ?loc}")
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .build();

        //read from file:
        let file = File::open("/Users/psbonte/Documents/Github/RoXi/examples/rsp/location_update_stream.nt").unwrap();

        for (i,lines) in io::BufReader::new(file).lines().enumerate(){
            let lines = lines.unwrap();
            let mut line = lines.split(" ");
            let triple = WindowTriple{s:line.next().unwrap().to_string(),
                p:line.next().unwrap().to_string(),
                o: line.next().unwrap().to_string(),};

            engine.add(triple,i);
        }
        engine.stop();
        thread::sleep(Duration::from_secs(2));
    }
    #[test]
    #[ignore]
    fn rsp_transitive_testp(){
        let ntriples_file = "";
        let rules = "@prefix test: <http://test/>.
 @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.
 {?x test:isIn ?y. ?y test:isIn ?z. }=>{?x test:isIn ?z.}";
        let function = Box::new(|r|println!("Bindings: {:?}",r));
        let result_consumer = ResultConsumer{function: Arc::new(function)};
        let r2r = Box::new(SimpleR2R {item: TripleStore::new()});
        let mut engine = RSPBuilder::new(10,2)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(ntriples_file)
            .add_syntax(Syntax::NTriples)
            .add_rules(rules)
            .add_query("Select * WHERE{ ?x <http://test/isIn> ?y}")
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .set_operation_mode(OperationMode::SingleThread)
            .build();
        for i in 0..20 {
            let triple = WindowTriple{s:format!("<http://test/{}>", i), p:"<http://test/isIn>".to_string(),o: format!("<http://test/{}>", i+1)};

            engine.add(triple,i);
        }
        engine.stop();

    }
}