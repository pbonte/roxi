extern crate oxigraph;

pub mod reasoningstore;

use oxigraph::MemoryStore;
use oxigraph::model::*;
use oxigraph::sparql::{QueryResults};
use oxigraph::io::GraphFormat;
use std::collections::HashMap;
use oxigraph::model::NamedNode;

use reasoningstore::{
    RuleIndex,
    ReasonerTriple,
    Rule,
    convert_to_query,
    ReasoningStore
};

fn main() {
    let mut reasoning_store = ReasoningStore::new();
    let timer = ::std::time::Instant::now();

// insertion
    let ex = NamedNode::new("http://example.com").unwrap();
    let quad = Quad::new(ex.clone(), ex.clone(), ex.clone(), None);
    reasoning_store.store.insert(quad.clone());
    // insertion
    let file = b"<http://example2.com/a> <http://rdf/type> <http://test.com/C1> .";
    reasoning_store.store.load_graph(file.as_ref(), GraphFormat::NTriples, &GraphName::DefaultGraph, None).unwrap();
    let file2 = b"<http://example2.com/b> <http://rdf/type> <http://test.com/C0> .";

    reasoning_store.store.load_graph(file2.as_ref(), GraphFormat::NTriples, &GraphName::DefaultGraph, None).unwrap();

    let b = BlankNode::new("s").unwrap();
    for i in 1..1000 {
        let head = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://rdf/type").unwrap()), o: NamedOrBlankNode::from(NamedNode::new(format!("http://test.com/C{}",i)).unwrap()) };
        //println!("{}", head.to_string());
        let body = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://rdf/type").unwrap()), o: NamedOrBlankNode::from(NamedNode::new(format!("http://test.com/C{}",(i-1))).unwrap()) };
        let mut body_rules = Vec::new();
        body_rules.push(body);
        let rule = Rule { body: body_rules, head: head };
        reasoning_store.add_rule(rule.clone());

    }
    println!("Done adding");
    reasoning_store.materialize();
    let elapsed = timer.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

// quad filter
    let results: Vec<Quad> = reasoning_store.store.quads_for_pattern(Some(ex.as_ref().into()), None, None, None).collect();
    assert_eq!(vec![quad], results);


//SPARQL query
//     if let QueryResults::Solutions( solutions) =  reasoning_store.query("SELECT * WHERE { ?s ?p ?o }").unwrap() {
//         //let re = extract_value(solutions);
//         //assert_eq!(solutions.next().unwrap().unwrap().get("s"), Some(&ex.into()));
//         for sol in solutions.into_iter(){
//             match sol{
//                 Ok(s) =>print!("{} {} {}\n", s.get("s").unwrap(),s.get("p").unwrap(),s.get("o").unwrap()),
//                 Err(_) =>print!("error"),
//             }
//
//         }
//         //print!("{}", solutions.next().unwrap().unwrap().get("s").unwrap());
//     }
}
// extern crate oxigraph;
//
// use oxigraph::MemoryStore;
// use oxigraph::model::*;
// use oxigraph::sparql::{QueryResults};
// use oxigraph::io::GraphFormat;
//
//
// // fn extract_value(mut solutions:QuerySolutionIter ) ->Option<&'static Term>{
// //     solutions.next().and_then(|s| s.and_then(|e| Ok(e.get("s"))))
// // }
// fn main()  {
//
//     let store = MemoryStore::new();
//
// // insertion
//     let ex = NamedNode::new("http://example.com").unwrap();
//     let quad = Quad::new(ex.clone(), ex.clone(), ex.clone(), None);
//     store.insert(quad.clone());
//     // insertion
//     let file = b"<http://example2.com> <http://example2.com> <http://example2.com> .";
//     store.load_graph(file.as_ref(), GraphFormat::NTriples, &GraphName::DefaultGraph, None).unwrap();
//
// // quad filter
//     let results: Vec<Quad> = store.quads_for_pattern(Some(ex.as_ref().into()), None, None, None).collect();
//     assert_eq!(vec![quad], results);
//
//
// // SPARQL query
//     if let QueryResults::Solutions( solutions) =  store.query("SELECT * WHERE { ?s ?p ?o }").unwrap() {
//         //let re = extract_value(solutions);
//         //assert_eq!(solutions.next().unwrap().unwrap().get("s"), Some(&ex.into()));
//         for sol in solutions.into_iter(){
//             match sol{
//                 Ok(s) =>print!("{} {} {}\n", s.get("s").unwrap(),s.get("p").unwrap(),s.get("o").unwrap()),
//                 Err(_) =>print!("error"),
//             }
//
//         }
//         //print!("{}", solutions.next().unwrap().unwrap().get("s").unwrap());
//     }
//
// }
