use std::rc::Rc;
use crate::{Binding, Rule, RuleIndex, Triple, TripleIndex, VarOrTerm, Encoder,TripleStore};
#[cfg(not(test))]
use log::{info, warn,trace, debug}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn, println as trace, println as debug};

pub struct BackwardChainer;

impl BackwardChainer {
    pub fn eval_backward(triple_index: &TripleIndex, rule_index: &RuleIndex, rule_head: &Triple) -> Binding {
        let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> = Self::find_subrules(rule_index, rule_head);
        let mut all_bindings = Binding::new();
        for (sub_rule, var_subs) in sub_rules.into_iter() {
            debug!("Backchainging rule: {:?}",TripleStore::decode_rule(&sub_rule));
            let mut rule_bindings = Binding::new();
            for rule_atom in &sub_rule.body {
                debug!("Matching body: {:?}",TripleStore::decode_triple(rule_atom));

                if let Some(result_bindings) = triple_index.query(rule_atom, None) {
                    debug!("   Found matching body: {:?}",TripleStore::decode_bindings(&result_bindings));

                    rule_bindings = rule_bindings.join(&result_bindings);

                }
                //recursive call
                let recursive_bindings = Self::eval_backward(triple_index, rule_index, rule_atom);
                rule_bindings.combine(recursive_bindings);
            }
            //rename variables
            let renamed = rule_bindings.rename(var_subs);
            all_bindings.combine(renamed);
        }
        all_bindings
    }
    //todo create index on rule heads
    pub fn find_subrules(rules_index: &RuleIndex, rule_head: &Triple) -> Vec<(Rc<Rule>, Vec<(usize, usize)>)> {
        let mut rule_matches = Vec::new();
        for rule in rules_index.rules.iter() {
            let head: &Triple = &rule.head;
            let mut var_names_subs: Vec::<(usize, usize)> = Vec::new();
            if Self::eval_triple_element(&head.s, &rule_head.s, &mut var_names_subs) &&
                Self::eval_triple_element(&head.p, &rule_head.p, &mut var_names_subs) &&
                Self::eval_triple_element(&head.o, &rule_head.o, &mut var_names_subs) {
                rule_matches.push((rule.clone(), var_names_subs));
            }
        }
        rule_matches
    }
    fn eval_triple_element(left: &VarOrTerm, right: &VarOrTerm, var_names_sub: &mut Vec<(usize, usize)>) -> bool {
        if let (VarOrTerm::Var(left_name), VarOrTerm::Var(right_name)) = (left, right) {
            var_names_sub.push((left_name.name, right_name.name));
            true
        } else {
            left.eq(right)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{BackwardChainer, Encoder, Syntax, Triple, TripleStore, VarOrTerm};

    #[test]
    fn test(){
        let triples = "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix : <http://www.some.com/>.
:sensor1 rdf:type :Sensor.
:sensor1 :observes :temp.
:temp rdf:type :Temp.
:obs rdf:type :Observation.
:obs :madeBySensor :sensor1.
:obs :observedProperty :temp.
";

        let rules ="@prefix : <http://www.some.com/>.
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
{?x rdf:type :Observation. ?x :madeBySensor ?y. ?y rdf:type :TempSensor}=>{?x rdf:type :TempObservation.}
{?x rdf:type :Sensor. ?x :observes ?y. ?y rdf:type :Temp}=>{?x rdf:type :TempSensor.}.
{?x rdf:type :TempObservation} => {?x rdf:type :EnvironmentObservation.}.
";

        let mut store = TripleStore::new();
        store.load_triples(triples, Syntax::Turtle);
        store.load_rules(rules);

        //backward head
        let backward_head = Triple::from("?x".to_string(),"<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),"<http://www.some.com/EnvironmentObservation>".to_string());
        let var_encoded= Encoder::add("x".to_string());
        let result_encoded = Encoder::add("<http://www.some.com/obs>".to_string());

        let  bindings = BackwardChainer::eval_backward(&store.triple_index, &store.rules_index, &backward_head);
        let result_bindings = HashMap::from([
            (var_encoded, Vec::from([result_encoded]))
        ]);

        assert_eq!(1,bindings.len());
        assert_eq!(result_bindings.get(&var_encoded), bindings.get(&var_encoded));
    }
    #[test]
    fn test_eval_backward_rule(){
        let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClass.\n\
            {?s a test:SubClass.}=>{?s a test:SubClass2.}\n
            {?s a test:SubClass2.?s test:hasRef ?b.?b test:hasRef ?c.?c a test:SubClass2.}=>{?s a test:SuperType.}";
        let mut store = TripleStore::from(data);
        let backward_head = Triple{s:VarOrTerm::new_var("?newVar".to_string()),p:VarOrTerm::new_term("a".to_string()),o:VarOrTerm::new_term("test:SuperType".to_string()), g: None};
        let var_encoded= Encoder::add("?newVar".to_string());
        let result_encoded = Encoder::add("<http://example2.com/a>".to_string());

        let  bindings = BackwardChainer::eval_backward(&store.triple_index, &store.rules_index, &backward_head);
        let result_bindings = HashMap::from([
            (var_encoded, Vec::from([result_encoded]))
        ]);
        assert_eq!(result_bindings.get(&12), bindings.get(&12));
    }
}

