use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
#[cfg(not(test))]
use log::{info, warn, trace, debug}; // Use log crate when building application
#[cfg(test)]
use std::{println as info, println as warn, println as trace, println as debug};use spargebra::Query;
use crate::rsp::windowing::{ContentGraph, CSPARQLWindow, Report, ReportStrategy, Tick, WindowTriple};
use crate::{Encoder, Syntax, Triple, TripleStore};
use crate::rsp::r2s::{Relation2StreamOperator, StreamOperator};
use crate::sparql::{Binding, eval_query, evaluate_plan, evaluate_plan_and_debug};
use crate::tripleindex::EncodedBinding;

mod windowing;
mod r2s;

struct RSPEngine{
    s2r: CSPARQLWindow,
    r2r: Arc<Mutex<TripleStore>>,
    r2r_consumere: Option<JoinHandle<()>>,
    r2s_consumer: ResultConsumer,
    r2s_operator: Arc<Mutex<Relation2StreamOperator>>
}
struct ResultConsumer{
    function: Arc<dyn Fn(Vec<Binding>) ->() + Send + Sync>
}
impl  RSPEngine{

    pub fn new(width: usize, slide: usize, tick: Tick, report_strategy: ReportStrategy, triples: &str, syntax: Syntax, rules: &str, query_str: &str, result_consumer: ResultConsumer, r2s: StreamOperator) -> RSPEngine{
        let mut report = Report::new();
        report.add(report_strategy);
        let mut window = CSPARQLWindow::new(width, slide, report, tick);
        let mut store = TripleStore::new();

        store.load_triples(triples, syntax);
        store.load_rules(rules);
        let query = Query::parse(query_str, None).unwrap();
        let mut engine = RSPEngine{s2r: window, r2r:  Arc::new(Mutex::new(store)), r2r_consumere: None, r2s_consumer: result_consumer, r2s_operator: Arc::new(Mutex::new(Relation2StreamOperator::new(r2s,0)))};
        let consumer = engine.s2r.register();
        engine.register_r2r(consumer, query);
        engine
    }
    fn register_r2r(&mut self,receiver: Receiver<ContentGraph>, query: Query){
        let mut consumer_temp = self.r2r.clone();
        let r2s_consumer = self.r2s_consumer.function.clone();
        let mut r2s_operator = self.r2s_operator.clone();
        let t = thread::spawn(move||{
            loop{
                match receiver.recv(){
                    Ok(mut content)=> {
                        debug!("Found graph {:?}", content);
                        content.into_iter().for_each(|t|{
                            if let Ok(mut store) = consumer_temp.lock(){
                                let encoded_triple = Triple::from(t.s,t.p,t.o,&mut store.encoder);
                                store.add(encoded_triple);
                                store.materialize();
                            }else{
                                println!("Unable to get lock!");
                            }
                        });
                        if let Ok(mut store) = consumer_temp.lock() {
                            let mut encoder = store.encoder.clone();
                            let plan = eval_query(&query, &store.triple_index, &mut encoder);
                            let iterator = evaluate_plan_and_debug(&plan, &store.triple_index, &mut encoder);
                            let r2s_result = r2s_operator.lock().unwrap().eval(iterator.collect(),content.get_last_timestamp_changed());
                            for result in r2s_result {
                                //println!("Query REsults: {:?}", result);

                                (r2s_consumer)(result);
                            }
                        }else{
                            println!("Unable to get lock!");
                        }

                    },
                    Err(_) => {
                        debug!("Shutting down!");
                        break;
                    }
                }
            }
            debug!("Exited loop");
        });
        self.r2r_consumere.replace(t);
    }
    pub fn add(&mut self, subject: String,
               property: String,
               object: String,
               ts: usize) {
        let triple = WindowTriple{s:subject, p:property,o: object};
        self.s2r.add_to_window(triple,ts);
    }
    pub fn stop(&mut self){
        self.s2r.stop();
        if let Some(thread) = self.r2r_consumere.take(){
            thread.join();
        }
    }
}
#[cfg(test)]
mod tests{
    use std::time::Duration;
    use super::*;

    #[test]
    fn rsp_integration(){
        let ntriples_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let rules = "@prefix test: <http://www.w3.org/test/>.\n{?x <http://test.be/hasVal> ?y. ?y <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> test:SuperType.}";
        let function = Box::new(|r|println!("Bindings: {:?}",r));
        let result_consumer = ResultConsumer{function: Arc::new(function)};
        let mut engine = RSPEngine::new(10,2,Tick::TimeDriven,ReportStrategy::OnWindowClose,ntriples_file,Syntax::NTriples,rules,"Select * WHERE{ ?s a <http://www.w3.org/test/SuperType>}", result_consumer,StreamOperator::RSTREAM);

        for i in 0..10 {

            engine.add(format!("s{}", i), "<http://test.be/hasVal>".to_string(), "<http://example.com/foo>".to_string(), i);
        }
        engine.stop();
        thread::sleep(Duration::from_secs(1));

    }
}