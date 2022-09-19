use std::collections::HashMap;
use crate::{Encoder, Rule as ReasonerRule, Triple, VarOrTerm};

use std::fmt::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

#[derive(Debug,  Clone)]
pub struct PrefixMapper{
    prefixes: HashMap<String,String>
}

impl PrefixMapper{
    pub fn new() -> PrefixMapper{
        PrefixMapper{prefixes:HashMap::new()}
    }
    pub fn add(&mut self, prefix: String, full_name:String){
        self.prefixes.insert(prefix,full_name);
    }
    pub fn expand(&self, prefixed:String) -> String{
        let mut split = prefixed.split(":");
        let vec: Vec<&str> = split.collect();
        if vec.len() >= 2 {
            let t = vec.get(0);
            if let Some(expanded_prefix) = self.prefixes.get(*vec.get(0).unwrap()) {
                let remainder_uri = *vec.get(1).unwrap();
                format!("{}{}", expanded_prefix, remainder_uri)
            }else{
                prefixed
            }
        }else {
            prefixed
        }
    }
}

#[derive(Parser)]
#[grammar = "parser/n3.pest"]
pub struct CSVParser;

fn parse_term(tp_term: Pair<Rule>) {
    match tp_term.as_rule(){
        Rule::Var=> println!("Var{:?}", tp_term.as_str()),
        Rule::Term=> println!("Term{:?}", tp_term.as_str()),

        Rule::EOI => (),
        _ => (),
    }
}

fn parse_tp(pair: Pairs<'_, Rule>, prefixes : &PrefixMapper, encoder: &mut Encoder) -> Triple{
    let mut subject_str="".to_string();
    let mut property_str = "".to_string();
    let mut object_str = "".to_string();
    for sub_rule in pair {
        match sub_rule.as_rule() {
            Rule::Subject => subject_str= prefixes.expand(sub_rule.as_str().to_string()),
            Rule::Property => property_str= prefixes.expand(sub_rule.as_str().to_string()),
            Rule::Object => object_str= prefixes.expand(sub_rule.as_str().to_string()),
            Rule::EOI => (),
            _ => (),
        }
    }
    Triple::from(subject_str,property_str,object_str, encoder)
}

pub fn parse(parse_string: &str, encoder: &mut Encoder) -> Result<Vec<ReasonerRule>,&'static str>{
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
                        //todo fix unneeded triple allocation
                        let mut head: Triple = Triple::from("?s".to_string(), "?p".to_string(), "?o".to_string(), encoder);
                        let mut body: Vec<Triple> = Vec::new();
                        for sub_rule in sub_rules {
                            match sub_rule.as_rule() {
                                Rule::Body => {
                                    let mut rules = sub_rule.into_inner();
                                    for rule in rules {
                                        body.push(parse_tp(rule.into_inner(), &prefix_mapper, encoder));
                                    }
                                },
                                Rule::Head => {
                                    head = parse_tp(sub_rule.into_inner().next().unwrap().into_inner(), &prefix_mapper, encoder);
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
        let mut encoder = Encoder::new();
        let rules = parse("@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n{?VaRr0 <http://test.be/pieter> ?lastVar. ?VaRr0 log:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}", &mut encoder).unwrap();
        println!("{:?}",rules);
        assert_eq!(rules.get(0).unwrap().body.len(), 2);
    }
    #[test]
    fn parse_multiple_prefixes() {
        let mut encoder = Encoder::new();

        let rules = parse("@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n @prefix log2: <http://www.w3.org/2000/10/swap/log2#>.\n {?VaRr0 <http://test.be/pieter> ?lastVar. ?VaRr0 log:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}", &mut encoder).unwrap();
        println!("{:?}",rules);
        assert_eq!(rules.get(0).unwrap().body.len(), 2);
    }
    #[test]
    fn parse_multiple_rules() {
        let mut encoder = Encoder::new();

        let rules = parse("@prefix log: <http://www.w3.org/2000/10/swap/log#>.\n @prefix log2: <http://www.w3.org/2000/10/swap/log2#>.\n {?VaRr0 <http://test.be/pieter> ?lastVar. ?VaRr0 log:type ?lastVar.}=>{?VaRr0 ssn:HasValue ?lastVar.}\n{?s <http://test.be/pieter> ?o.}=>{?s ssn:HasValue ?o.}",&mut encoder).unwrap();
        println!("{:?}",rules);
        assert_eq!(rules.len(), 2);
    }
    #[test]
    fn parse_multiple_rulese_ending_with_dot() {
        let mut encoder = Encoder::new();

        let rules = parse("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n@prefix : <http://eulersharp.sourceforge.net/2009/12dtb/test#>.\n{?V0 rdf:type :N0} => {?V0 rdf:type :N1}.\n{?V0 rdf:type :N1} => {?V0 rdf:type :N2}.", &mut encoder).unwrap();
        println!("{:?}",rules);
        assert_eq!(rules.len(), 2);

    }
}