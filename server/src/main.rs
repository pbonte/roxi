extern crate oxigraph;
extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate roxi;
extern crate minimal;
extern crate env_logger;
use minimal::{Rule, Triple, TripleStore, TermImpl, VarOrTerm, Encoder};
use minimal::ruleindex::RuleIndex;
use minimal::tripleindex::TripleIndex;
// use oxigraph::sparql::{QueryResults};
// use roxi::reasoningstore::ReasoningStore;
// use std::fs::{File, read_to_string};
// use std::io::{BufReader, Read};
// use clap::Parser;
// use env_logger::Env;
// use oxigraph::model::Triple;
// use roxi::reasoningstore::rule::Rule;
//
//
// #[derive(Parser, Debug)]
// #[clap(author, version, about, long_about = None)]
// struct Args {
//     /// File path to the ABox (in TTL format)
//     #[clap(short, long)]
//     abox: String,
//
//     /// File path to the TBox (in TTL format)
//     #[clap(short, long)]
//     tbox: String,
//
//     /// SPARQL query to be executed
//     #[clap(short, long)]
//     query: String,
//
//     /// Trace of reasoning process
//     #[clap(short, long)]
//     trace: Option<bool>,
// }

fn main(){
    let timer = ::std::time::Instant::now();
    let mut rules = Vec::new();
    let mut encoder = Encoder::new();
    let max_depth = 1000000;
    for i in 0..max_depth{
        let rule = Rule{head: Triple{s:VarOrTerm::newVar("s".to_string(), &mut encoder),p:VarOrTerm::newTerm("http://test".to_string(), &mut encoder),o:VarOrTerm::newTerm(format!("U{}", i+1), &mut encoder)},
            body: Vec::from([Triple{s:VarOrTerm::newVar("s".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm(format!("U{}",i),&mut encoder)}])};
        let rule2 = Rule{head: Triple{s:VarOrTerm::newVar("s".to_string(), &mut encoder),p:VarOrTerm::newTerm("http://test".to_string(), &mut encoder),o:VarOrTerm::newTerm(format!("J{}", i+1), &mut encoder)},
            body: Vec::from([Triple{s:VarOrTerm::newVar("s".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm(format!("U{}",i),&mut encoder)}])};
        let rule3 = Rule{head: Triple{s:VarOrTerm::newVar("s".to_string(), &mut encoder),p:VarOrTerm::newTerm("http://test".to_string(), &mut encoder),o:VarOrTerm::newTerm(format!("Q{}", i+1), &mut encoder)},
            body: Vec::from([Triple{s:VarOrTerm::newVar("s".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm(format!("U{}",i),&mut encoder)}])};
        rules.push(rule);
        rules.push(rule2);
        rules.push(rule3);
    }

    let content =  Vec::from([Triple{s:VarOrTerm::newTerm("sTerm".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm("U0".to_string(),&mut encoder)}]);
    let mut rules_index = RuleIndex::new();
    for rule in rules.iter(){
        rules_index.add(rule);
    }
    let mut triple_index = TripleIndex::new();
    content.into_iter().for_each(|t| triple_index.add(t));
    let query = Triple{s:VarOrTerm::newVar("s".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm(format!("U{}",max_depth),&mut encoder)};

    let mut store = TripleStore{rules:Vec::new(), rules_index , triple_index, encoder };

    store.materialize();
    let elapsed = timer.elapsed();

    let result = store.query(&query, None);

    println!("Processed in: {:.2?}", elapsed);

    println!("Store size: {:?}", store.len());
}
// fn main_old() {
//     let args = Args::parse();
//
//     let timer = ::std::time::Instant::now();
//     let f = File::open(args.abox).unwrap();
//     let reader = BufReader::new(f);
//     if let Some(true) = args.trace {
//         env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
//     }
//
//     println!("Loading data ABox and TBox");
//     let mut reasoning_store = ReasoningStore::new();
//     if args.tbox.ends_with("ttl"){
//         let f2 = File::open(args.tbox).unwrap();
//         let reader2 = BufReader::new(f2);
//         reasoning_store.load_tbox(reader2);
//     }else{
//         let file_content = read_to_string(args.tbox).unwrap();
//         reasoning_store.parse_and_add_rule(&file_content);
//     }
//     reasoning_store.load_abox(reader);
//     let elapsed = timer.elapsed();
//
//     println!("Data Loaded in: {:.2?}", elapsed);
//
//     println!("ABox Size: {}", reasoning_store.len_abox());
//
//     println!("Starting materialization");
//     let timer2 = ::std::time::Instant::now();
//     reasoning_store.materialize();
//     let elapsed2 = timer2.elapsed();
//     println!("Materialization Time: {:.2?}", elapsed2);
//     //SPARQL query
//     let q2: String = args.query;
//
//     println!("Results for query: {}:", q2);
//     if let QueryResults::Solutions(solutions) = reasoning_store.store.query(&q2).unwrap() {
//         for sol in solutions.into_iter() {
//             match sol {
//                 Ok(s) => s.iter().for_each(|b| println!("{:?}", b)),
//                 Err(_) => print!("error"),
//             }
//         }
//     }
//     println!("Size Materialized Store: {}", reasoning_store.store.len().unwrap());
// }
