use crate::{BackwardChainer, Binding, Encoder, QueryEngine, Reasoner, Rule, RuleIndex, SimpleQueryEngine, Triple, TripleIndex, TripleStore, VarOrTerm};
#[cfg(not(test))]
use log::{info, warn,trace}; // Use log crate when building application
use std::fmt::Write;

#[cfg(test)]
use std::{println as info, println as warn, println as trace};
use std::rc::Rc;
use crate::utils::Utils;

pub struct DRed{
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index : TripleIndex,
    pub encoder: Encoder,
    reasoner: Reasoner,
}

impl DRed{
    fn new() -> Self{
        Self{rules: Vec::new(), rules_index: RuleIndex::new(), triple_index: TripleIndex::new(), encoder: Encoder::new(), reasoner: Reasoner{} }
    }
    pub fn from(data:&str) -> Self{
        let triple_store = TripleStore::from(&data);
        Self{rules: triple_store.rules, rules_index: triple_store.rules_index , triple_index: triple_store.triple_index, encoder: triple_store.encoder,  reasoner: Reasoner{} }
    }
    pub fn add(&mut self, triple: Triple){
        trace!{"Adding triple: {:?}", Utils::decode_triple(&self.encoder,&triple) }
        self.triple_index.add(triple);
    }
    pub fn add_ref(&mut self, triple: Rc<Triple>){
        trace!{"Adding triple: {:?}", Utils::decode_triple(&self.encoder,triple.as_ref()) }
        self.triple_index.add_ref(triple);
    }
    pub fn remove_ref(&mut self, triple: Rc<Triple>){
        //println!("{:?}",self.encoder);

        trace!{"Removing triple: {:?}", Utils::decode_triple(&self.encoder,triple.as_ref()) }
        // over delete
        let mut over_deletion = Vec::new();
        let mut stack = Vec::from([triple.as_ref().clone()]);

        while let Some(current_triple) = stack.pop(){
            println!("Investigating deletion {:?}",Utils::decode_triple(&self.encoder,&current_triple));

            let matching_rules = self.rules_index.find_match(&current_triple);
            let matching_rules: Vec<Rule> = matching_rules.clone().into_iter().flat_map(|r| Reasoner::substitute_rule(&current_triple, r)).collect();
            matching_rules.iter().map(|r|Utils::decode_rule(&self.encoder,r)).for_each(|r|println!("Matching rules {:?}",r));

            let delete_triples = Reasoner::infer_rule_heads(&self.triple_index, None, matching_rules);
            delete_triples.into_iter().for_each(|t| {
                println!("Marked head for deletion {:?}",Utils::decode_triple(&self.encoder,&t));
                if ! over_deletion.contains(&t){
                    stack.push(t.clone());
                    over_deletion.push(t);
                }
            });

        }

        over_deletion.iter().map(|t|Utils::decode_triple(&self.encoder,t)).for_each(|t|println!("Overdeleted {:?}",t));
        // delete overdeletion
        over_deletion.iter().for_each(|t|self.triple_index.remove_ref(t));
        // delete E-
        self.triple_index.remove_ref(&triple);

        let mut test = ("123",true);
        let mut ref_test = &mut test;
        ref_test.1=false;
        let delete_list : Vec<(Triple,bool)>= over_deletion.into_iter().map(|t|(t,false)).collect();
        // Rederivation step
        let mut delete_num = delete_list.len()+1;
        let mut prev_delete_num = delete_num+1;
        while delete_num < prev_delete_num{
            prev_delete_num = delete_num;
            for (delete_triple, mut delete_status) in  &delete_list {
                if !delete_status {
                    println!("Trying redirive {:?}", Utils::decode_triple(&self.encoder, &delete_triple));

                    let matched_rules = Self::find_rules_by_head(&self.rules_index, &delete_triple);
                    for (matched_rule, bindings) in matched_rules {
                        println!("\t matched rule {:?}", Utils::decode_rule(&self.encoder, &matched_rule));

                        println!("\tBindings {:?}", bindings);
                        let substitute_rule = Reasoner::substitute_rule_body_with_binding(&matched_rule, &bindings);
                        substitute_rule.iter().for_each(|r| println!("\t subsitute rule_item {:?}", Utils::decode_triple(&self.encoder, r)));
                        if let Some(_) = SimpleQueryEngine::query(&self.triple_index, &substitute_rule, None) {
                            if ! self.triple_index.contains(delete_triple) {
                                println!("\t Reinsert {:?}", Utils::decode_triple(&self.encoder, &delete_triple));
                                self.triple_index.add(delete_triple.clone());
                                delete_num -= 1;
                                delete_status = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

    }
    //todo create index on rule heads
    pub(crate) fn find_rules_by_head(rules_index: &RuleIndex, head_triple: &Triple) -> Vec<(Rc<Rule>, Binding)> {
        let mut rule_matches = Vec::new();
        for rule in rules_index.rules.iter() {
            let head: &Triple = &rule.head;
            let mut binding = Binding::new();
            if Self::eval_triple_element(&head.s, &head_triple.s, &mut binding) &&
                Self::eval_triple_element(&head.p, &head_triple.p, &mut binding) &&
                Self::eval_triple_element(&head.o, &head_triple.o, &mut binding) {
                rule_matches.push((rule.clone(), binding));
            }
        }
        rule_matches
    }
    //todo check if code can be reused
    fn eval_triple_element(left: &VarOrTerm, right: &VarOrTerm, bindings: &mut Binding) -> bool {
        if let (VarOrTerm::Var(left_name), VarOrTerm::Term(right_name)) = (left, right) {
            bindings.add(&left_name.name, right_name.iri);
            true
        } else {
            left.eq(right)
        }
    }

    pub fn materialize(&mut self) -> Vec<Triple>{
        self.reasoner.materialize(&mut self.triple_index, &self.rules_index)
    }
}
mod test{
    use std::rc::Rc;
    use crate::dred::DRed;
    use crate::{Triple, VarOrTerm};
    use crate::utils::Utils;

    #[test]
    fn test(){
        let data=":john :teaches :math.\n\
                :peter :teaches :math.\n\
                :john :teaches :physics.\n\
            {?s :teaches ?y.}=>{?s a :Person.}\n\
            {?s :teaches ?y.}=>{?y a :Course.}\n\
            {?s a :TA.}=>{?s a :Person.}\n\
            {?s a :Person.?s :teaches ?y.?y a :Course.}=>{?s a :TA.}";
        let mut dred = DRed::from(data);
        let inferred = dred.materialize();
        inferred.iter().for_each(|t|println!("{:?}",Utils::decode_triple(&dred.encoder,t)));
        println!("{:?}", inferred);
        assert_eq!(9, dred.triple_index.len());

        let remove_triple = Triple{s:VarOrTerm::new_term(":john".to_string(), &mut dred.encoder),p:VarOrTerm::new_term(":teaches".to_string(), &mut dred.encoder),o:VarOrTerm::new_term(":math".to_string(), &mut dred.encoder), g: None};

        dred.remove_ref(Rc::new(remove_triple));
        assert_eq!(8, dred.triple_index.len());

    }
}