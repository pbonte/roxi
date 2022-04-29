

use std::fmt::Error;
use oxigraph::model::NamedOrBlankNode;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use crate::reasoningstore::triple::{PrefixMapper, ReasonerTriple};
use crate::reasoningstore::rule::Rule as ReasonerRule;


#[derive(Parser)]
#[grammar = "parsing/n3.pest"]
pub struct CSVParser;

fn parse_term(tp_term: Pair<Rule>) {
    match tp_term.as_rule(){
        Rule::Var=> println!("Var{:?}", tp_term.as_str()),
        Rule::Term=> println!("Term{:?}", tp_term.as_str()),

        Rule::EOI => (),
        _ => (),
    }
}

fn parse_tp(pair: Pairs<'_, Rule>, prefixes : &PrefixMapper) -> ReasonerTriple{
    let mut subject_str="";
    let mut property_str = "";
    let mut object_str = "";
    for sub_rule in pair {
        match sub_rule.as_rule() {
            Rule::Subject => subject_str= sub_rule.as_str(),
            Rule::Property => property_str= sub_rule.as_str(),
            Rule::Object => object_str= sub_rule.as_str(),
            Rule::EOI => (),
            _ => (),
        }
    }
    ReasonerTriple::new_from_prefixes(subject_str.to_string(),property_str.to_string(),object_str.to_string(),prefixes)
}

pub fn parse(parse_string: &str) -> Result<Vec<ReasonerRule>,&str>{
    let mut rules = Vec::new();
    let mut prefix_mapper = PrefixMapper::new();
    let mut unparsed = CSVParser::parse(Rule::document, parse_string).expect("Unable to read")
        .next();
    match unparsed {
        None => return Err("Parsing failed"),
        Some(parsed) => {
            for line in parsed.into_inner() {
                //println!("{:?}",line);
                match line.as_rule() {
                    Rule::Prefix => {
                        let mut name = line.into_inner();
                        let mut prefix_name = "";
                        let mut prefix_iri = "";
                        for prefix_sub in name {
                            match prefix_sub.as_rule() {
                                Rule::PrefixIdentifier => prefix_name = prefix_sub.as_str(),
                                Rule::Iri => prefix_iri = prefix_sub.as_str(),
                                Rule::EOI => (),
                                _ => (),
                            }
                        }
                        prefix_mapper.add(prefix_name.to_string(), prefix_iri.to_string());
                    }
                    Rule::rule => {
                        let mut sub_rules = line.into_inner();
                        let mut head: ReasonerTriple = ReasonerTriple::new("?s".to_string(), "?p".to_string(), "?o".to_string());
                        let mut body: Vec<ReasonerTriple> = Vec::new();
                        for sub_rule in sub_rules {
                            match sub_rule.as_rule() {
                                Rule::Body => {
                                    let mut rules = sub_rule.into_inner();
                                    for rule in rules {
                                        body.push(parse_tp(rule.into_inner(), &prefix_mapper));
                                    }
                                },
                                Rule::Head => {
                                    head = parse_tp(sub_rule.into_inner().next().unwrap().into_inner(), &prefix_mapper);
                                },

                                Rule::EOI => (),
                                _ => (),
                            }
                        }
                        rules.push(ReasonerRule { body: body, head: head })
                    }
                    // Rule::Var => {
                    //     println!("Var{}", line.as_str());
                    // }
                    Rule::EOI => (),
                    _ => println!("not Found {}", line.as_str()),
                }
            }
            return Ok(rules);
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn parse_tp() {
        let rules = parse("@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n{?VaRr0 <http://test.be/pieter> ?lastVar. ?VaRr0 log:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}").unwrap();
        println!("{:?}",rules);
        assert_eq!(rules.get(0).unwrap().body.len(), 2);
    }
    #[test]
    fn parse_multiple_prefixes() {
        let rules = parse("@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n @prefix log2: <http://www.w3.org/2000/10/swap/log2#>.\n {?VaRr0 <http://test.be/pieter> ?lastVar. ?VaRr0 log:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}").unwrap();
        println!("{:?}",rules);
        assert_eq!(rules.get(0).unwrap().body.len(), 2);
    }
    #[test]
    fn parse_multiple_rules() {
        let rules = parse("@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n @prefix log2: <http://www.w3.org/2000/10/swap/log2#>.\n {?VaRr0 <http://test.be/pieter> ?lastVar. ?VaRr0 log:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}\n{?s <http://test.be/pieter> ?o.}=>{?s ssn:HasValue ?o.}").unwrap();
        println!("{:?}",rules);
        assert_eq!(rules.len(), 2);
    }
}
