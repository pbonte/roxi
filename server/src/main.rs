extern crate oxigraph;
extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate roxi;
extern crate env_logger;

use oxigraph::sparql::{QueryResults};
use roxi::reasoningstore::ReasoningStore;
use std::fs::{File, read_to_string};
use std::io::{BufReader, Read};
use clap::Parser;
use env_logger::Env;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to the ABox (in TTL format)
    #[clap(short, long)]
    abox: String,

    /// File path to the TBox (in TTL format)
    #[clap(short, long)]
    tbox: String,

    /// SPARQL query to be executed
    #[clap(short, long)]
    query: String,
}


fn main() {
    let args = Args::parse();

    let timer = ::std::time::Instant::now();
    let f = File::open(args.abox).unwrap();
    let reader = BufReader::new(f);

    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    println!("Loading data ABox and TBox");
    let mut reasoning_store = ReasoningStore::new();
    if args.tbox.ends_with("ttl"){
        let f2 = File::open(args.tbox).unwrap();
        let reader2 = BufReader::new(f2);
        reasoning_store.load_tbox(reader2);
    }else{
        let file_content = read_to_string(args.tbox).unwrap();
        reasoning_store.parse_and_add_rule(&file_content);
    }
    reasoning_store.load_abox(reader);
    let elapsed = timer.elapsed();

    println!("Data Loaded in: {:.2?}", elapsed);

    println!("ABox Size: {}", reasoning_store.len_abox());

    println!("Starting materialization");
    let timer2 = ::std::time::Instant::now();
    reasoning_store.materialize();
    let elapsed2 = timer2.elapsed();
    println!("Materialization Time: {:.2?}", elapsed2);
    //SPARQL query
    let q2: String = args.query;

    println!("Results for query: {}:", q2);
    if let QueryResults::Solutions(solutions) = reasoning_store.store.query(&q2).unwrap() {
        for sol in solutions.into_iter() {
            match sol {
                Ok(s) => s.iter().for_each(|b| println!("{:?}", b)),
                Err(_) => print!("error"),
            }
        }
    }
    println!("Size Materialized Store: {}", reasoning_store.store.len().unwrap());
}

