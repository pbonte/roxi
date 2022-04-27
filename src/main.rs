extern crate oxigraph;
extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod reasoningstore;
mod n3_parser;

use oxigraph::sparql::{QueryResults};
use reasoningstore::ReasoningStore;
use std::fs::File;
use std::io::BufReader;
use clap::Parser;


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

fn main(){
    let rules = n3_parser::parse("@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n{?VaRr0 rdf:type ?lastVar. ?VaRr0 rdf:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}");
    println!("{:?}",rules);
}

fn main_old() {
    let args = Args::parse();

    let timer = ::std::time::Instant::now();

    let f = File::open(args.abox).unwrap();
    let reader = BufReader::new(f);
    let f2 = File::open(args.tbox).unwrap();
    let reader2 = BufReader::new(f2);

    println!("Loading data ABox and TBox");
    let mut reasoning_store = ReasoningStore::new();
    reasoning_store.load_tbox(reader2);
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
    if let QueryResults::Solutions(solutions) = reasoning_store.reasoning_store.query(&q2).unwrap() {
        for sol in solutions.into_iter() {
            match sol {
                Ok(s) => s.iter().for_each(|b| println!("{:?}", b)),
                Err(_) => print!("error"),
            }
        }
    }
    println!("Size Materialized Store: {}", reasoning_store.reasoning_store.len());
}

