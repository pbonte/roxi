use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use log::debug;
use spargebra::Query;
use crate::rsp::windowing::{ContentGraph, CSPARQLWindow, Report, ReportStrategy, Tick};
use crate::{Syntax, Triple, TripleStore};

mod windowing;

struct ConsumerInner{
    data: Mutex<TripleStore>
}
struct RSPEngine{
    s2r: CSPARQLWindow,
    r2r: Arc<ConsumerInner>,
    query: Query
}

impl RSPEngine{
    pub fn new(width: usize, slide: usize, tick: Tick, report_strategy: ReportStrategy, triples: &str, syntax: Syntax,rules: &str, query_str: &str) -> RSPEngine{
        let mut report = Report::new();
        report.add(report_strategy);
        let mut window = CSPARQLWindow::new(width, slide, report, tick);
        let mut store = TripleStore::new();
        store.load_triples(triples, syntax);
        store.load_rules(rules);
        let query = Query::parse(query_str, None).unwrap();
        let mut engine = RSPEngine{s2r: window, r2r:  Arc::new(ConsumerInner{data: Mutex::new(store)}), query};
        let consumer = engine.s2r.register();
        //engine.register_r2r(consumer);
        engine
    }
    // fn register_r2r(&self,receiver: Receiver<ContentGraph>){
    //     let mut consumer_temp = self.r2r.clone();
    //     thread::spawn(move||{
    //         loop{
    //             match receiver.recv(){
    //                 Ok(content)=> {
    //                     debug!("Found graph {:?}", content);
    //                     content.into_iter().for_each(|t|{
    //                         consumer_temp.data.lock().unwrap().add(t);
    //                     })
    //
    //                 },
    //                 Err(_) => {
    //                     debug!("Shutting down!");
    //                     break;
    //                 }
    //             }
    //         }
    //     });
    // }
    pub fn add(&mut self, triple: Triple, ts: usize) {
        self.s2r.add_to_window(triple,ts);
    }
}
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn rsp_integration(){
        let ntriples_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let rules = "@prefix test: <http://www.w3.org/test>.\n{?x <http://test.be/hasVal> ?y. ?y <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> test:SuperType.}";
        let mut engine = RSPEngine::new(10,2,Tick::TimeDriven,ReportStrategy::OnWindowClose,ntriples_file,Syntax::NTriples,rules,"Select * WHERE{ ?s ?p ?o}");

        // for i in 0..10 {
        //     let triple = Triple::from(format!("s{}", i), "<http://test.be/hasVal>".to_string(), "<http://example.com/foo>".to_string(), &mut engine.r2r.encoder);
        //     engine.add(triple, i);
        // }
    }
}