use std::collections::HashMap;
use std::rc::Rc;
use crate::{Rule, Triple};


pub struct RuleIndex   {
    pub(crate) rules: Vec<Rc<Rule>>,
    spo:Vec<Rc<Rule>>,
    s:HashMap<usize,  Vec<Rc<Rule>>>,
    p:HashMap<usize, Vec<Rc<Rule>>>,
    o:HashMap<usize, Vec<Rc<Rule>>>,
    sp:HashMap<usize,  HashMap<usize,Vec<Rc<Rule>>>>,
    po:HashMap<usize,  HashMap<usize,Vec<Rc<Rule>>>>,
    so:HashMap<usize,  HashMap<usize,Vec<Rc<Rule>>>>,
}



impl  RuleIndex {
    pub fn len(&self) -> usize {
        self.spo.len() + self.s.len() + self.o.len() + self.p.len() +
            self.sp.len() + self.po.len() + self.so.len()
    }
    pub fn new() -> RuleIndex{
        RuleIndex{
            rules: Vec::new(),
            s:HashMap::new(),
            p:HashMap::new(),
            o:HashMap::new(),
            so:HashMap::new(),
            po:HashMap::new(),
            sp:HashMap::new(),
            spo:Vec::new()}
    }
    fn add_rc(&mut self, rule: Rc<Rule>){
        self.rules.push(rule.clone());
        for Triple{s ,p,o}  in rule.body.iter(){
            //s match
            if s.is_term() && p.is_var() && o.is_var(){
                if !self.s.contains_key(&s.to_encoded()){
                    self.s.insert(s.to_encoded(), Vec::new());
                }
                if let Some(mut rules) = self.s.get_mut(&s.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
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
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
            }
            //o match
            if s.is_var() && p.is_var() && o.is_term(){
                if !self.o.contains_key(&o.to_encoded()){
                    self.o.insert(o.to_encoded(), Vec::new());
                }
                //self.o.get_mut(&o.to_string()).unwrap().push(rule.clone());
                if let Some(mut rules) = self.o.get_mut(&o.to_encoded()){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
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
                    if !rules.contains(&rule) {rules.push(rule.clone())};
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
                    if !rules.contains(&rule) {rules.push(rule.clone())};
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
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
            }
            //spo
            if s.is_var() && p.is_var() && o.is_var() {
                //self.spo.push(rule.clone());
                if !self.spo.contains(&rule) {self.spo.push(rule.clone())};

            }

        }
    }
    pub fn add(&mut self, rule: Rule){
        let clone_rule = Rc::new(rule);
        self.add_rc(clone_rule);
    }
    pub fn add_ref(&mut self, rule:  & Rule ){
        let clone_rule = Rc::new(rule.clone());
        self.add_rc(clone_rule);
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
