extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate minimal as roxi;
extern crate env_logger;

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use roxi::ruleindex::RuleIndex;
use roxi::tripleindex::TripleIndex;
use roxi::TripleStore;


use std::fs::{File, read_to_string};
use std::io::{BufReader, Read};
use clap::Parser;
use env_logger::Env;
use spargebra::Query;
use roxi::encoding::Encoder;
use roxi::parser::Syntax;
use roxi::parser::Parser as TripleParser;
use roxi::sparql::{eval_query, evaluate_plan_and_debug};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to the ABox (in TTL format)
    #[clap(short, long)]
    abox: String,

    /// File path to the TBox (in TTL format)
    #[clap(short, long)]
    tbox: String,

    // /// SPARQL query to be executed
    // #[clap(short, long)]
    // query: String,

    /// Trace of reasoning process
    #[clap(short, long)]
    trace: Option<bool>,
}


fn main() {
    let args = Args::parse();

    let timer = ::std::time::Instant::now();
    if let Some(true) = args.trace {
        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    }

    println!("Loading data ABox in NTriples");
    let mut store = TripleStore::new();
    let file_content = read_to_string(args.abox).unwrap();
    store.load_triples(&file_content,Syntax::NTriples);

    println!("Loading data Rulse in N3");

    let rules = read_to_string(args.tbox).unwrap();
    store.load_rules(&rules);
    let elapsed = timer.elapsed();

    println!("Data Loaded in: {:.2?}", elapsed);

    println!("ABox Size: {}", store.len());

    println!("Starting materialization");
    let timer2 = ::std::time::Instant::now();
    store.materialize();
    let elapsed2 = timer2.elapsed();
    println!("Materialization Time: {:.2?}", elapsed2);
    //SPARQL query
    // let q2: String = args.query;

    // println!("Results for query: {}:", q2);
    // let query = Query::parse(&q2, None).unwrap();
    // let plan = eval_query(&query, &store.triple_index, &mut encoder);
    // let iterator = evaluate_plan_and_debug(&plan, &store.triple_index, &mut encoder);
    // for result in iterator{
    //     println!("Bindings {:?}:", result);
    // }

    println!("Content: \n{:?}", store.content_to_string());
    println!("Size Materialized Store: {}", store.len());
}

