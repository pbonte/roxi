use std::rc::Rc;
use crate::{Binding, Rule, RuleIndex, Triple, TripleIndex, VarOrTerm};

pub struct BackwardChainer;

impl BackwardChainer {
    pub(crate) fn eval_backward(triple_index: &TripleIndex, rule_index: &RuleIndex, rule_head: &Triple) -> Binding {
        let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> = Self::find_subrules(rule_index, rule_head);
        let mut all_bindings = Binding::new();
        for (sub_rule, var_subs) in sub_rules.into_iter() {
            let mut rule_bindings = Binding::new();
            for rule_atom in &sub_rule.body {
                if let Some(result_bindings) = triple_index.query(rule_atom, None) {
                    rule_bindings = rule_bindings.join(&result_bindings);
                    //recursive call
                    let recursive_bindings = Self::eval_backward(triple_index, rule_index, rule_atom);
                    rule_bindings.combine(recursive_bindings);
                }
            }
            //rename variables
            let renamed = rule_bindings.rename(var_subs);
            all_bindings.combine(renamed);
        }
        all_bindings
    }
    //todo create index on rule heads
    pub(crate) fn find_subrules(rules_index: &RuleIndex, rule_head: &Triple) -> Vec<(Rc<Rule>, Vec<(usize, usize)>)> {
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

