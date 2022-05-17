use std::collections::HashMap;
use crate::{Rule, Triple};


pub struct RuleIndex <'a>  {
    spo:Vec<&'a Rule>,
    s:HashMap<usize,  Vec<&'a Rule>>,
    p:HashMap<usize, Vec<&'a Rule>>,
    o:HashMap<usize,  Vec<&'a Rule>>,
    sp:HashMap<usize,  HashMap<usize,Vec<&'a Rule>>>,
    po:HashMap<usize,  HashMap<usize,Vec<&'a Rule>>>,
    so:HashMap<usize,  HashMap<usize,Vec<&'a Rule>>>,
}



impl<'a>  RuleIndex<'a> {
    pub fn len(&self) -> usize {
        self.spo.len() + self.s.len() + self.o.len() + self.p.len() +
            self.sp.len() + self.po.len() + self.so.len()
    }
    pub fn new() -> RuleIndex<'a>{
        RuleIndex{s:HashMap::new(),
            p:HashMap::new(),
            o:HashMap::new(),
            so:HashMap::new(),
            po:HashMap::new(),
            sp:HashMap::new(),
            spo:Vec::new()}
    }
    pub fn add(&mut self, rule:  &'a Rule ){
        for Triple{s ,p,o}  in rule.body.iter(){
            //s match
            if s.is_term() && p.is_var() && o.is_var(){
                if !self.s.contains_key(&s.to_encoded()){
                    self.s.insert(s.to_encoded(), Vec::new());
                }
                if let Some(mut rules) = self.s.get_mut(&s.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule)};
                }
                // self.s.get(&s.to_string()).unwrap().push(rule.clone());
            }
            //p match
            if s.is_var() && p.is_term() && o.is_var(){
                if !self.p.contains_key(&p.to_encoded()){
                    self.p.insert(p.to_encoded(), Vec::new());
                }
                //self.p.get_mut(&p.to_string()).unwrap().push(rule.clone());
                if let Some(mut rules) = self.p.get_mut(&p.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule)};
                }
            }
            //o match
            if s.is_var() && p.is_var() && o.is_term(){
                if !self.o.contains_key(&o.to_encoded()){
                    self.o.insert(o.to_encoded(), Vec::new());
                }
                //self.o.get_mut(&o.to_string()).unwrap().push(rule.clone());
                if let Some(mut rules) = self.o.get_mut(&o.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule)};
                }
            }
            //sp
            if s.is_term() && p.is_term() && o.is_var(){
                if !self.sp.contains_key(&s.to_encoded()){
                    self.sp.insert(s.to_encoded(), HashMap::new());
                }
                if !self.sp.get(&s.to_encoded()).unwrap().contains_key(&p.to_encoded()){
                    self.sp.get_mut(&s.to_encoded()).unwrap().insert(p.to_encoded(), Vec::new());
                }
                //self.sp.get_mut(&sp_str).unwrap().push(rule.clone());
                if let Some(mut rules) = self.sp.get_mut(&s.to_encoded()).unwrap().get_mut(&p.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule)};
                }
            }
            //so
            if s.is_term() && p.is_var() && o.is_term(){
                if !self.so.contains_key(&s.to_encoded()){
                    self.so.insert(s.to_encoded(), HashMap::new());
                }
                if !self.so.get(&s.to_encoded()).unwrap().contains_key(&o.to_encoded()){
                    self.so.get_mut(&s.to_encoded()).unwrap().insert(o.to_encoded(), Vec::new());
                }
                //self.sp.get_mut(&sp_str).unwrap().push(rule.clone());
                if let Some(mut rules) = self.so.get_mut(&s.to_encoded()).unwrap().get_mut(&o.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule)};
                }
            }
            //po
            if s.is_var() && p.is_term() && o.is_term(){
                if !self.po.contains_key(&p.to_encoded()){
                    self.po.insert(p.to_encoded(), HashMap::new());
                }
                if !self.po.get(&p.to_encoded()).unwrap().contains_key(&o.to_encoded()){
                    self.po.get_mut(&p.to_encoded()).unwrap().insert(o.to_encoded(), Vec::new());
                }
                //self.sp.get_mut(&sp_str).unwrap().push(rule.clone());
                if let Some(mut rules) = self.po.get_mut(&p.to_encoded()).unwrap().get_mut(&o.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule)};
                }
            }
            //spo
            if s.is_var() && p.is_var() && o.is_var() {
                //self.spo.push(rule.clone());
                if !self.spo.contains(&rule) {self.spo.push(rule)};

            }

        }
    }

    pub fn find_match(&self, triple: &Triple) ->Vec<&Rule>{
        let mut matched_triples: Vec<&Rule> = Vec::new();
        //check s
        if let Some(rule) = self.s.get(&triple.s.to_encoded()){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check p
        if let Some(rule) = self.p.get(&triple.p.to_encoded()){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check o
        if let Some(rule) = self.o.get(&triple.o.to_encoded()){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check so
        if let Some(s_rules) = self.so.get(&triple.s.to_encoded()){
            if let Some(rules) = s_rules.get(&triple.o.to_encoded()) {
                rules.iter().for_each(|r| matched_triples.push(r));
            }
        }
        //check po
        if let Some(p_rules) = self.po.get(&triple.p.to_encoded()){
            if let Some(rules) = p_rules.get(&triple.o.to_encoded()) {
                rules.iter().for_each(|r| matched_triples.push(r));
            }
        }
        //check sp
        if let Some(s_rules) = self.sp.get(&triple.s.to_encoded()){
            if let Some(rules) = s_rules.get(&triple.p.to_encoded()) {
                rules.iter().for_each(|r| matched_triples.push(r));
            }
        }
        self.spo.iter().for_each(|r| matched_triples.push(r));

        matched_triples
    }
}
