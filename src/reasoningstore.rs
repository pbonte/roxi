extern crate oxigraph;
pub mod triple;
pub mod ruleindex;
pub mod rule;
    use oxigraph::MemoryStore;
    use oxigraph::model::*;
    use oxigraph::sparql::{QueryResults, Query};
    use oxigraph::io::{ DatasetFormat};
    use oxigraph::model::NamedOrBlankNode;
    use rio_turtle::{TurtleParser, TurtleError};
    use rio_api::parser::TriplesParser;
    use std::io::BufRead;
    use std::io;
use std::rc::Rc;

use triple::ReasonerTriple;
use crate::reasoningstore::rule::Rule;
use crate::reasoningstore::ruleindex::RuleIndex;


fn find_rule_match<'a>(quad:&'a Quad, rules: &'a Vec<Rule>) -> Vec<&'a Rule>{
        let mut matched_triples: Vec<&Rule> = Vec::new();
        for rule in rules.iter(){
            for ReasonerTriple{s ,p,o}  in rule.body.iter(){
                let mut match_triple = true;

                match s{
                    NamedOrBlankNode::NamedNode(node_iri) if !quad.subject.to_string().eq(&node_iri.to_string())=> match_triple=false,
                    _ => match_triple = true,
                }
                match p{
                    NamedOrBlankNode::NamedNode(node_iri) if !quad.predicate.to_string().eq(&node_iri.to_string())=> match_triple=false,
                    _ => match_triple = true,
                }
                match o{
                    NamedOrBlankNode::NamedNode(node_iri) if !quad.object.to_string().eq(&node_iri.to_string())=>  match_triple=false,
                    _ => match_triple = true,
                }
                if match_triple {
                    matched_triples.push(rule);
                }
            }
        }
        matched_triples
    }
    pub fn convert_to_query(rule : &Rule) -> Query{
        let body_string:String = rule.body.iter().map(|r| r.to_string()).collect();
        let query_string = format!("CONSTRUCT {{ {} }} WHERE {{ {} }}",rule.head.to_string(), body_string);
        Query::parse(&query_string,None).unwrap()
    }

    pub struct ReasoningStore{
        pub store: MemoryStore,
        pub(crate) reasoning_store: MemoryStore,
        rules: Vec<Rc<Rule>>,
        rules_index: RuleIndex
    }
impl ReasoningStore {
    pub fn new() -> ReasoningStore{
        ReasoningStore{store: MemoryStore::new(), reasoning_store: MemoryStore::new(),
            rules:Vec::new(), rules_index: RuleIndex::new()}
    }
    pub fn load_abox(&self, reader: impl BufRead)-> Result<(), io::Error>{
        self.store.load_dataset(reader,DatasetFormat::TriG, None)
    }
    pub fn load_tbox(&mut self, reader: impl BufRead){
        let rdf_subclass = String::from("<http://www.w3.org/2000/01/rdf-schema#subClassOf>");
        TurtleParser::new(reader, None).parse_all(&mut |triple| {

            if triple.predicate.to_string().eq(&rdf_subclass){
                let str_len = triple.object.to_string().len();
                let object_str =  &triple.object.to_string()[1..str_len-1];
                let str_len = triple.subject.to_string().len();
                let subject_str =  &triple.subject.to_string()[1..str_len-1];
                if let Ok(named_subject) = NamedNode::new(subject_str){
                    let body = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap()), o:  NamedOrBlankNode::from(named_subject)};
                    if let Ok(named) = NamedNode::new(object_str) {
                        let head = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap()), o: NamedOrBlankNode::from(named) };
                        let mut body_rules = Vec::new();
                        body_rules.push(body);
                        let rule = Rc::new(Rule { body: body_rules, head });
                        self.add_rule(rule.clone());
                    }
                }
            }
            Ok(()) as Result<(), TurtleError>
        }).unwrap();

    }
    pub fn len_rules(&self) -> usize{
        self.rules_index.len()
    }
    pub fn len_abox(&self) -> usize{
        self.store.len()
    }
    pub fn add_rule(&mut self,rule:Rc<Rule>){
        self.rules.push(rule.clone());
        self.rules_index.add(rule.clone());
    }
    pub fn materialize(&self) {
        let mut tripe_queue: Vec<Quad> = self.store.iter().collect();
        // iterate over all triples
        let mut queue_iter = 0;
        while queue_iter < tripe_queue.len() {
            let mut temp_triples = Vec::new();
            let quad = tripe_queue.get(queue_iter).unwrap();
            if !self.reasoning_store.contains(quad) {
                self.reasoning_store.insert(quad.clone());

                //let matched_rules = find_rule_match(&quad, &rules); // without indexing
                let matched_rules = self.rules_index.find_match(&quad);
                // find matching rules
                for matched_rule in matched_rules.into_iter() {
                    let q = convert_to_query(matched_rule);
                    if let QueryResults::Graph(solutions) = self.reasoning_store.query(q).unwrap() {
                        for sol in solutions.into_iter() {
                            match sol {
                                Ok(s) => temp_triples.push(Quad::new(s.subject.clone(), s.predicate.clone(), s.object.clone(), None)),
                                _ => (),
                            }
                        }
                    }
                }
            }
            queue_iter += 1;
            temp_triples.iter().for_each(|t| tripe_queue.push(t.clone()));
        }
    }
}
