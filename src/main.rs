extern crate oxigraph;

pub mod reasoningstore;
use oxigraph::model::*;
use oxigraph::sparql::{QueryResults};
use oxigraph::io::{GraphFormat, DatasetFormat};
use oxigraph::model::NamedNode;
use reasoningstore::ReasoningStore;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use crate::reasoningstore::rule::Rule;
use crate::reasoningstore::triple::ReasonerTriple;


fn main(){
    let timer = ::std::time::Instant::now();


    let f = File::open("/Users/psbonte/Downloads/challenge/tbox.ttl").unwrap();
    let mut reader = BufReader::new(f);

    let f2 = File::open("/Users/psbonte/Downloads/challenge/abox.ttl").unwrap();
    let mut reader2 = BufReader::new(f2);

    println!("Loading data");
    let mut reasoning_store = ReasoningStore::new();
    let timer = ::std::time::Instant::now();
    let result = reasoning_store.load_tbox(reader);
    let result2 = reasoning_store.load_abox(reader2);
    println!("Data Loaded");
    let elapsed = timer.elapsed();

    println!("Elapsed: {:.2?}", elapsed);

    println!("{}",reasoning_store.len_abox());
    println!("{}",reasoning_store.len_rules());

    println!("Starting materialization");
    let timer2 = ::std::time::Instant::now();
    reasoning_store.materialize();
    let elapsed2 = timer2.elapsed();
    println!("Elapsed: {:.2?}", elapsed2);
//SPARQL query
    if let QueryResults::Solutions( solutions) =  reasoning_store.reasoning_store.query("SELECT * WHERE { <http://example.com/condition0> a ?type }").unwrap() {
        //let re = extract_value(solutions);
        //assert_eq!(solutions.next().unwrap().unwrap().get("s"), Some(&ex.into()));
        for sol in solutions.into_iter(){
            match sol{
                Ok(s) =>print!("{} \n", s.get("type").unwrap()),
                Err(_) =>print!("error"),
            }

        }
        //print!("{}", solutions.next().unwrap().unwrap().get("s").unwrap());
    }
    println!("{}",reasoning_store.reasoning_store.len());
}

fn test_main() {
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
        let rule = Rc::new(Rule { body: body_rules, head: head });
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
