use crate::{Binding, QueryEngine, RuleIndex, SimpleQueryEngine, TermImpl, Triple, TripleIndex, VarOrTerm};
#[cfg(not(test))]
use log::{info, warn,trace}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn, println as trace};

pub struct Reasoner;

impl Reasoner{
    pub fn materialize(&mut self, triple_index: &mut TripleIndex, rules_index: &RuleIndex) -> Vec<Triple>{
        let mut inferred = Vec::new();
        let mut counter = 0;
        while(counter < triple_index.triples.len()){
            let process_quad = triple_index.get(counter).unwrap();
            //trace!("Processing: {:?}",decode_triple(process_quad));
            //let matching_rules = self.find_matching_rules(process_quad);
            let matching_rules = rules_index.find_match(process_quad);
            trace!("Found Rules: {:?}",matching_rules);
            let mut new_triples = Vec::new();
            for rule in matching_rules{
                if let Some(temp_bindings) = SimpleQueryEngine::query(triple_index, &rule.body,Some(counter + 1)) {
                    let new_heads = self.subsititue_rule_head(&rule.head, &temp_bindings);
                    for new_head in new_heads {
                        new_triples.push(new_head);
                    }
                }
            }
            for triple in new_triples{
                if !triple_index.contains(&triple) {
                    //trace!("Inferred: {:?}",self.decode_triple(&triple));
                    inferred.push(triple.clone());
                    triple_index.add(triple);
                }
            }
            counter+=1;
        }

        inferred
    }
    fn subsititue_rule_head(&self, head: &Triple, binding: &Binding) ->Vec<Triple>{
        let mut new_heads = Vec::new();
        let mut s: &usize;
        let mut p: &usize;
        let mut o: &usize;
        for result_counter in 0..binding.len(){
            match &head.s{
                VarOrTerm::Var(s_var)=> s = binding.get(&s_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(s_term)=> s = &s_term.iri
            }
            match &head.p{
                VarOrTerm::Var(p_var)=> p = binding.get(&p_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(p_term)=> p = &p_term.iri
            }
            match &head.o{
                VarOrTerm::Var(o_var)=> o = binding.get(&o_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(o_term)=> o = &o_term.iri
            }
            new_heads.push(Triple{s:VarOrTerm::Term(TermImpl{iri:s.clone()}),p:VarOrTerm::Term(TermImpl{iri:p.clone()}),o:VarOrTerm::Term(TermImpl{iri:o.clone()})})
        }

        new_heads
    }
}