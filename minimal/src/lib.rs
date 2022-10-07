extern crate core;
pub mod ruleindex;
pub mod tripleindex;
pub mod imars_window;
pub mod imars_reasoner;
pub mod bindings;
pub mod triples;
pub mod encoding;
pub mod queryengine;
pub mod reasoner;
pub mod parser;
pub mod backwardchaining;
pub mod csprite;
pub mod observer;
pub mod time_window;
pub mod pipeline;
pub mod sparql;
pub mod dred;
pub mod utils;
pub mod rsp;
extern crate pest;
#[macro_use]
extern crate pest_derive;
use crate::ruleindex::RuleIndex;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::tripleindex::TripleIndex;
use std::fmt::Write;
use crate::bindings::Binding;

#[cfg(not(test))]
use log::{info, warn,trace}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn, println as trace};
use crate::backwardchaining::BackwardChainer;
use crate::encoding::Encoder;
use crate::parser::{Parser,Syntax};

use crate::queryengine::{QueryEngine, SimpleQueryEngine};
use crate::reasoner::Reasoner;
use crate::triples::{Rule, TermImpl, Triple, VarOrTerm}; // Workaround to use prinltn! for logs.



pub struct TripleStore{
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index : TripleIndex,
    pub encoder: Encoder,
    pub reasoner: Reasoner
}



impl TripleStore {
    pub fn new() -> TripleStore{
        TripleStore{rules: Vec::new(), rules_index: RuleIndex::new(), triple_index: TripleIndex::new(), encoder: Encoder::new(), reasoner: Reasoner{ } }
    }
    pub fn from(data:&str) -> TripleStore{
        let mut encoder = Encoder::new();
        let (content, rules) = Parser::parse(data.to_string(),&mut encoder);
        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add_ref(rule);
        }
        TripleStore{rules:rules, rules_index , triple_index, encoder,reasoner: Reasoner{ } }
    }
    pub fn add(&mut self, triple: Triple){
        trace!{"Adding triple: {:?}", self.decode_triple(&triple) }
        self.triple_index.add(triple);
    }
    pub fn add_ref(&mut self, triple: Rc<Triple>){
        trace!{"Adding triple: {:?}", self.decode_triple(triple.as_ref()) }
        self.triple_index.add_ref(triple);
    }
    pub fn remove_ref(&mut self, triple: Rc<Triple>){
        trace!{"Removing triple: {:?}", self.decode_triple(triple.as_ref()) }
        self.triple_index.remove_ref(&triple);
    }
    pub(crate) fn add_rules(&mut self, rules: Vec<Rule>) {
        rules.into_iter().for_each(|rule|self.rules_index.add(rule));
    }
    pub fn len(&self) -> usize{
        self.triple_index.len()
    }
    fn decode_triple(&self, triple:  &Triple) -> String {
        let s = self.encoder.decode(&triple.s.to_encoded()).unwrap();
        let p = self.encoder.decode(&triple.p.to_encoded()).unwrap();
        let o = self.encoder.decode(&triple.o.to_encoded()).unwrap();
        format!("{} {} {}",s,p,o)
    }
    pub fn materialize(&mut self) -> Vec<Triple>{
        self.reasoner.materialize(&mut self.triple_index,&self.rules_index)
    }

    //Backward chaining




    ////



    pub fn decode_rule(rule: &Rule, encoder:&Encoder) -> String{
        let mut res = String::new();
        let decoded_head = Self::decode_triples(&[rule.head.clone()],encoder);
        let decoded_body : String = rule.body.iter().map(|t|Self::decode_triples(&[t.clone()],encoder)).collect();
        write!(&mut res, "{{{}}}=>{{{}}}.\n", decoded_body, decoded_head).unwrap();
        res
    }
    pub fn decode_rules(rules: &[Rule], encoder:&Encoder) -> String{
        let mut res = String::new();
        for rule in rules {
            let decoded_head = Self::decode_triples(&[rule.head.clone()],encoder);
            let decoded_body : String = rule.body.iter().map(|t|Self::decode_triples(&[t.clone()],encoder)).collect();
            write!(&mut res, "{{{}}}=>{{{}}}.\n", decoded_body, decoded_head).unwrap();
        }
        res
    }
    pub fn decode_triples(triples: &[Triple], encoder: &Encoder) -> String {
        let mut res = String::new();
        for triple in triples {
            let decoded_s = encoder.decode(&triple.s.to_encoded()).unwrap();
            let decoded_p = encoder.decode(&triple.p.to_encoded()).unwrap();
            let decoded_o = encoder.decode(&triple.o.to_encoded()).unwrap();

            write!(&mut res, "{} {} {}.\n", decoded_s, decoded_p, decoded_o).unwrap();
        }
        res
    }
    pub fn content_to_string(&mut self) -> String{
        let content = &self.triple_index.triples;
        let encoder = &self.encoder;
        TripleStore::decode_triples(content,encoder)
    }

    pub fn load_triples(&mut self, data: &str, syntax: Syntax) -> Result<(),&'static str>{
        match Parser::parse_triples(data,&mut self.encoder,syntax) {
            Ok(triples) =>{triples.into_iter().for_each(|t| self.triple_index.add(t));
                Ok(())
            },
            Err(err)=> Err(err)
        }
    }
    pub fn load_rules(&mut self, data: &str) -> Result<(),&'static str>{
        match Parser::parse_rules(data,&mut self.encoder) {
            Ok(rules) =>{
                rules.into_iter().for_each(|t| self.rules_index.add(t));
                Ok(())
            },
            Err(err)=> Err(err)
        }
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::{Rule, Triple, TripleStore, TermImpl, VarOrTerm, RuleIndex, TripleIndex, Encoder, SimpleQueryEngine, QueryEngine, Parser, BackwardChainer};
    use crate::reasoner::Reasoner;

    #[test]
    fn test_parse(){
        let data=":a a :C0.\n\
            {?a a :C0}=>{?a a :C1}\n\
            {?a a :C1}=>{?a a :C2}\n\
            {?a a :C2}=>{?a a :C3}";

        let mut store = TripleStore::from(data);

        let mat = store.materialize();
        println!("Length: {:?}", store.len());
        println!("Length Mat: {:?}", mat.len());
    }


    #[test]
    fn test_store() {
        let timer = ::std::time::Instant::now();
        let mut rules = Vec::new();
        let mut encoder = Encoder::new();
        let max_depth = 5;
        for i in 0..max_depth{
            let rule = Rule{head: Triple{s:VarOrTerm::new_var("s".to_string(), &mut encoder),p:VarOrTerm::new_term("http://test".to_string(), &mut encoder),o:VarOrTerm::new_term(format!("U{}", i+1), &mut encoder), g: None},
                body: Vec::from([Triple{s:VarOrTerm::new_var("s".to_string(), &mut encoder),p:VarOrTerm::new_term("http://test".to_string(), &mut encoder),o:VarOrTerm::new_term(format!("U{}", i), &mut encoder), g: None}])};
            rules.push(rule);
        }

        let content =  Vec::from([Triple{s:VarOrTerm::new_term("sTerm".to_string(), &mut encoder),p:VarOrTerm::new_term("http://test".to_string(), &mut encoder),o:VarOrTerm::new_term("U0".to_string(), &mut encoder), g: None}]);
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add_ref(rule);
        }
        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let query = Triple{s:VarOrTerm::new_var("s".to_string(), &mut encoder),p:VarOrTerm::new_term("http://test".to_string(), &mut encoder),o:VarOrTerm::new_term(format!("U{}", max_depth), &mut encoder), g: None};

        let mut store = TripleStore{rules:Vec::new(), rules_index , triple_index, encoder,reasoner: Reasoner{ } };

        store.materialize();
        let elapsed = timer.elapsed();

        let result = SimpleQueryEngine::query(&store.triple_index, &Vec::from([query]), None);

        println!("Processed in: {:.2?}", elapsed);
        println!("Result: {:?}", result);

    }


    #[test]
    fn test_incomplete_rule_match(){
        let data=":a in :b.\n\
            {?a in ?b. ?b in ?c}=>{?a in ?c.}";

        let mut store = TripleStore::from(data);
        assert_eq!(1,store.len());
        store.materialize();
        assert_eq!(1,store.len());

    }
    #[test]
    fn test_no_var_query(){
        let data=":a in :b.\n\
            {:a in :b}=>{:a in :c}";

        let mut store = TripleStore::from(data);
        assert_eq!(1,store.len());
        store.materialize();
        assert_eq!(2,store.len());

    }
    #[test]
    fn test_single_rule() {
        let data=":a a :A.\n\
            {?a a :A}=>{?a a :B}";

        let mut store = TripleStore::from(data);
        assert_eq!(1,store.len());
        store.materialize();
        assert_eq!(2,store.len());

    }
    #[test]
    fn test_multiple_rule() {
        let data=":a a :A.\n\
            {?a a :A}=>{?a a :B}\n\
            {?a a :B}=>{?a a :C}";

        let mut store = TripleStore::from(data);
        assert_eq!(1,store.len());
        store.materialize();
        assert_eq!(3,store.len());
    }
    #[test]
    fn test_join_rule() {
        let data=":a a :A.\n\
            :a in :b.\n\
            {?a a :A.?a in ?o}=>{?a a :B}";

        let mut store = TripleStore::from(data);
        assert_eq!(2,store.len());
        store.materialize();
        assert_eq!(3,store.len());
    }
    #[test]
    fn test_long_join_rule() {
        let data=":a a :A.\n\
            :a in :b.\n\
            :b in :c.\n\
            :c a :A.\n\
            {?a a :A.?a in ?o.?o in ?o2.?o2 a :A}=>{?a a :B}";

        let mut store = TripleStore::from(data);
        assert_eq!(4,store.len());
        store.materialize();
        assert_eq!(5,store.len());
    }
    #[test]
    fn test_transitive_rule(){
        let mut data = "{?a in ?b.?b in ?c}=>{?a in ?c}\n".to_owned();
        for i in 0..10{
            data += format!(":{} in :{}.\n",i+1,i).as_str();
        }
        let mut store = TripleStore::from(data.as_str());
        assert_eq!(10,store.len());
        store.materialize();
        assert_eq!(55,store.len());
    }
    #[test]
    fn test_hierarchy(){
        let max_depth = 10;
        let mut data = ":a a :U0\n".to_owned();
        for i in 0..max_depth {
            data += format!("{{?a a :U{}}}=>{{?a a :U{}}}\n", i, i + 1).as_str();
            data += format!("{{?a a :U{}}}=>{{?a a :J{}}}\n", i, i + 1).as_str();
            data += format!("{{?a a :U{}}}=>{{?a a :Q{}}}\n", i, i + 1).as_str();
        }
        println!("{}", data);
        let mut store = TripleStore::from(data.as_str());
        let inferred = store.materialize();
        println!("Triples: {:?}", store.len());
        assert_eq!(3 * max_depth, inferred.len());
    }
    #[test]
    #[ignore]
    fn test_rdf_hierarchy(){
        let max_depth = 10;
        let mut data = ":a a :U0\n\
                        {?a :subClassOf ?b.?b :subClassOf ?c}=>{?a :subClassOf ?c}\n".to_owned();
        for i in 0..max_depth {
            data += format!(":U{} :subClassOf :U{}.\n", i, i + 1).as_str();
            data += format!(":U{} :subClassOf :J{}.\n", i, i + 1).as_str();
            data += format!(":U{} :subClassOf :Q{}.\n", i, i + 1).as_str();
        }
        let mut store = TripleStore::from(data.as_str());
        let inferred = store.materialize();
        println!("Triples: {:?}", store.len());
        assert_eq!(3 * max_depth, inferred.len());    }
    // #[test]
    // fn test_eval_backward_multiple_rules(){
    //     let mut store = ReasoningStore::new();
    //     store.parse_and_add_rule("@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n \
    //     {?s rdf:type test:SubClass.}=>{?s rdf:type test:SuperType.}\n\
    //     {?s rdf:type test:SubClass2.}=>{?s rdf:type test:SuperType.}");
    //     store.load_abox( b"<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .".as_ref());
    //     store.load_abox( b"<http://example2.com/c> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass2> .".as_ref());
    //
    //     // diff variable names
    //     let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());
    //     let  bindings = store.eval_backward( &backward_head);
    //     let mut result_bindings: Binding = Binding::new();
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/a".to_string()).unwrap()));
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/c".to_string()).unwrap()));
    //
    //     assert_eq!(result_bindings, bindings);
    // }
    // #[test]
    // fn test_eval_backward_nested_rules(){
    //     let mut store = ReasoningStore::new();
    //     store.parse_and_add_rule("@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n \
    //     {?s rdf:type test:SubClass. ?s test:hasRef ?o. ?o rdf:type test:SubClass2.}=>{?s rdf:type test:SuperType.}\n\
    //     {?q rdf:type test:SubClassTemp.}=>{?q rdf:type test:SubClass2.}");
    //     store.load_abox( b"<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .".as_ref());
    //     store.load_abox( b"<http://example2.com/b> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClassTemp> .".as_ref());
    //     store.load_abox( b"<http://example2.com/a> <http://www.test.be/test#hasRef> <http://example2.com/b> .".as_ref());
    //
    //     // diff variable names
    //     let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());
    //     let  bindings = store.eval_backward( &backward_head);
    //     let mut result_bindings: Binding = Binding::new();
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/a".to_string()).unwrap()));
    //
    //     assert_eq!(result_bindings, bindings);
    // }
}
