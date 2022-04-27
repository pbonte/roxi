extern crate pest;
#[macro_use]
extern crate pest_derive;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(Parser)]
#[grammar = "csv.pest"]
pub struct CSVParser;

fn parse_term(tp_term: Pair<Rule>){
    match tp_term.as_rule(){
        Rule::Var=> println!("Var{:?}", tp_term.as_str()),
        Rule::Term=> println!("Term{:?}", tp_term.as_str()),

        Rule::EOI => (),
        _ => println!("Not found{:?}", tp_term),
    }
}

fn parse_tp(pair: Pairs<'_, Rule>){
    let mut subject_str="";
    let mut property_str = "";
    let mut object_str = "";
    for sub_rule in pair {
        match sub_rule.as_rule() {
            Rule::Subject => subject_str= sub_rule.as_str(),
            Rule::Property => property_str= sub_rule.as_str(),
            Rule::Object => object_str= sub_rule.as_str(),
            Rule::EOI => (),
            _ => println!("Not found{:?}", sub_rule),
        }
    }
}

fn main() {
    let unparsed = CSVParser::parse(Rule::document, "@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n{?VaRr0 rdf:type ?lastVar. ?VaRr0 rdf:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}").expect("Unable to read")
        .next().unwrap();
    println!("{}",unparsed);
    for line in unparsed.into_inner() {
        //println!("{:?}",line);
        match line.as_rule() {
            Rule::Prefix => {
                let mut name = line.into_inner();
                for prefix_sub in name{
                    match prefix_sub.as_rule(){
                        Rule::PrefixIdentifier =>  println!("prefix {:?}", prefix_sub.as_str()),
                        Rule::Iri => println!("iri {:?}", prefix_sub.as_str()),
                        Rule::EOI => (),
                        _ => println!("not Found {}", prefix_sub.as_str()),
                    }

                }

            }
            Rule::rule => {
                let mut sub_rules = line.into_inner();
                for sub_rule in sub_rules{
                    match sub_rule.as_rule(){
                        Rule::Body =>  {println!("body {:?}", sub_rule.as_str()); parse_tp(sub_rule.into_inner())},
                        Rule::Head => { println!("head {:?}", sub_rule.as_str()); parse_tp(sub_rule.into_inner().next().unwrap().into_inner())},

                        Rule::EOI => (),
                        _ => println!("not Found {}", sub_rule.as_str()),
                    }
                }
            }
            Rule::Var => {

                println!("Var{}",line.as_str());
            }
            Rule::EOI => (),
            _ => println!("not Found {}",line.as_str()),
        }
    }
}
