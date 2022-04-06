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
    convert_to_query
};

fn main() {
    let store = MemoryStore::new();
    let reasoning_store = MemoryStore::new();
    let timer = ::std::time::Instant::now();
    let mut rule_index = RuleIndex::new();
    // let mut rule_index = RuleIndex{s:HashMap::new(),
    //             p:HashMap::new(),
    //             o:HashMap::new(),
    //             so:HashMap::new(),
    //             po:HashMap::new(),
    //             sp:HashMap::new(),
    //             spo:Vec::new()};
// insertion
    let ex = NamedNode::new("http://example.com").unwrap();
    let quad = Quad::new(ex.clone(), ex.clone(), ex.clone(), None);
    store.insert(quad.clone());
    // insertion
    let file = b"<http://example2.com/a> <http://rdf/type> <http://test.com/C1> .";
    store.load_graph(file.as_ref(), GraphFormat::NTriples, &GraphName::DefaultGraph, None).unwrap();
    let file2 = b"<http://example2.com/b> <http://rdf/type> <http://test.com/C0> .";

    store.load_graph(file2.as_ref(), GraphFormat::NTriples, &GraphName::DefaultGraph, None).unwrap();

    let mut rules = Vec::new();
    let b = BlankNode::new("s").unwrap();
    for i in 1..1000 {
        let head = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://rdf/type").unwrap()), o: NamedOrBlankNode::from(NamedNode::new(format!("http://test.com/C{}",i)).unwrap()) };
        //println!("{}", head.to_string());
        let body = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://rdf/type").unwrap()), o: NamedOrBlankNode::from(NamedNode::new(format!("http://test.com/C{}",(i-1))).unwrap()) };
        let mut body_rules = Vec::new();
        body_rules.push(body);
        let rule = Rule { body: body_rules, head: head };
        rule_index.add(rule.clone());
        let q = convert_to_query(&rule);
        rules.push(rule);
    }
    println!("Done adding");
    let mut tripe_queue: Vec<Quad> = store.iter().collect();
    // iterate over all triples
    let mut queue_iter = 0;
    while queue_iter < tripe_queue.len() {
        let mut temp_triples = Vec::new();
        let  quad = tripe_queue.get(queue_iter).unwrap();
        if !reasoning_store.contains(quad) {
            reasoning_store.insert(quad.clone());
            //println!("Processing quad{:?}", quad);

             //let matched_rules = find_rule_match(&quad, &rules);
            let matched_rules = rule_index.find_match(&quad);
            // find matching rules
            //println!("Matching rules {:?}", matched_rules);
            for matched_rule in matched_rules.into_iter() {
                let q = convert_to_query(matched_rule);
                if let QueryResults::Graph(solutions) = reasoning_store.query(q).unwrap() {
                    //let re = extract_value(solutions);
                    //assert_eq!(solutions.next().unwrap().unwrap().get("s"), Some(&ex.into()));
                    for sol in solutions.into_iter() {
                        match sol {
                            Ok(s)  => temp_triples.push(Quad::new(s.subject.clone(), s.predicate.clone(), s.object.clone(), None)),
                            _ => (),
                        }
                    }
                    //print!("{}", solutions.next().unwrap().unwrap().get("s").unwrap());
                }
            }
        }
        queue_iter+=1;
        temp_triples.iter().for_each(|t| tripe_queue.push(t.clone()));
    }
    let elapsed = timer.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

// quad filter
    let results: Vec<Quad> = store.quads_for_pattern(Some(ex.as_ref().into()), None, None, None).collect();
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
