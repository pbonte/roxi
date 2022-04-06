extern crate datafrog;
extern crate oxigraph;

use datafrog::Iteration;

use oxigraph::MemoryStore;
use oxigraph::model::*;
use oxigraph::sparql::{QueryResults};
use oxigraph::io::GraphFormat;
fn main() {
    let store = MemoryStore::new();
    let timer = ::std::time::Instant::now();

    // Create a new iteration context, ...
    let mut iteration = Iteration::new();

    // .. some variables, ..
    let type_var = iteration.variable::<(&str, &str)>("types");
    let subClass_var = iteration.variable::<(&str, &str)>("subclass");

    // .. load them with some initial values, ..
    // Make space for input data.
    let mut types = Vec::new();
    let mut subclassOf = Vec::new();
    types.push(("B", "a"));

    subclassOf.push(("A", "B"));

    subclassOf.push(("B", "C"));
    subclassOf.push(("C", "D"));
    subclassOf.push(("D", "E"));


    type_var.insert(types.into());
    subClass_var.insert(subclassOf.into());


    // .. and then start iterating rules!
    while iteration.changed() {
        // nodes(a,c)  <-  nodes(b,a), edges(b,c)
        type_var.from_join(&type_var, &subClass_var, |_b, &a, &c| (c, a));
    }

    // extract the final results.
    let reachable = type_var.complete();
    let reachable2 = subClass_var.complete();
    println!(
        "{:?}\tComputation complete (nodes_final: {})",
        timer.elapsed(),
        reachable.len()
    );
    println!("{:?}", reachable.elements);
    println!("{:?}", reachable2.elements);
    //     let store = MemoryStore::new();
//
// insertion
    let ex = NamedNode::new("http://example.com").unwrap();
    let quad = Quad::new(ex.clone(), ex.clone(), ex.clone(), None);
    store.insert(quad.clone());
    // insertion
    let file = b"<http://example2.com/a> <http://rdf/type> <http://example2.com/A> .";
    store.load_graph(file.as_ref(), GraphFormat::NTriples, &GraphName::DefaultGraph, None).unwrap();

    // load inferred triples
    for (ind_type, ind) in reachable.elements.into_iter(){
        let file = format!("<http://example2.com/{}> <http://rdf/type> <http://example2.com/{}> .",ind,ind_type);
        store.load_graph(file.as_ref(), GraphFormat::NTriples, &GraphName::DefaultGraph, None).unwrap();
    }

// quad filter
    let results: Vec<Quad> = store.quads_for_pattern(Some(ex.as_ref().into()), None, None, None).collect();
    assert_eq!(vec![quad], results);


// SPARQL query
    if let QueryResults::Solutions( solutions) =  store.query("SELECT * WHERE { ?s ?p ?o }").unwrap() {
        //let re = extract_value(solutions);
        //assert_eq!(solutions.next().unwrap().unwrap().get("s"), Some(&ex.into()));
        for sol in solutions.into_iter(){
            match sol{
                Ok(s) =>print!("{} {} {}\n", s.get("s").unwrap(),s.get("p").unwrap(),s.get("o").unwrap()),
                Err(_) =>print!("error"),
            }

        }
        //print!("{}", solutions.next().unwrap().unwrap().get("s").unwrap());
    }
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
