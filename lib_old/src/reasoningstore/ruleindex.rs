use std::collections::HashMap;
use std::rc::Rc;
use crate::reasoningstore::rule::Rule;
use crate::reasoningstore::triple::ReasonerTriple;
use oxigraph::model::Quad;
pub struct RuleIndex {
    spo:Vec<Rc<Rule>>,
    s:HashMap<String,  Vec<Rc<Rule>>>,
    p:HashMap<String, Vec<Rc<Rule>>>,
    o:HashMap<String,  Vec<Rc<Rule>>>,
    sp:HashMap<String,  Vec<Rc<Rule>>>,
    po:HashMap<String,  Vec<Rc<Rule>>>,
    so:HashMap<String,  Vec<Rc<Rule>>>,
}

impl RuleIndex {
    pub fn len(&self) -> usize {
        self.spo.len() + self.s.len() + self.o.len() + self.p.len() +
            self.sp.len() + self.po.len() + self.so.len()
    }
}

impl  RuleIndex {
    pub fn new() -> RuleIndex{
        RuleIndex{s:HashMap::new(),
            p:HashMap::new(),
            o:HashMap::new(),
            so:HashMap::new(),
            po:HashMap::new(),
            sp:HashMap::new(),
            spo:Vec::new()}
    }
    pub fn add(&mut self, rule:  Rc<Rule> ){
        for ReasonerTriple{s ,p,o}  in rule.body.iter(){
            //s match
            if s.is_named_node() && p.is_blank_node() && o.is_blank_node(){
                if !self.s.contains_key(&s.to_string()){
                    self.s.insert(s.to_string(),Vec::new());
                }
                if let Some(mut rules) = self.s.get_mut(&s.to_string()){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
                // self.s.get(&s.to_string()).unwrap().push(rule.clone());
            }
            //p match
            if s.is_blank_node() && p.is_named_node() && o.is_blank_node(){
                if !self.p.contains_key(&p.to_string()){
                    self.p.insert(p.to_string(),Vec::new());
                }
                //self.p.get_mut(&p.to_string()).unwrap().push(rule.clone());
                if let Some(mut rules) = self.p.get_mut(&p.to_string()){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
            }
            //o match
            if s.is_blank_node() && p.is_blank_node() && o.is_named_node(){
                if !self.o.contains_key(&o.to_string()){
                    self.o.insert(o.to_string(),Vec::new());
                }
                //self.o.get_mut(&o.to_string()).unwrap().push(rule.clone());
                if let Some(mut rules) = self.o.get_mut(&o.to_string()){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
            }
            //sp
            if s.is_named_node() && p.is_named_node() && o.is_blank_node(){
                let sp_str = format!("{}{}",s.to_string(),p.to_string());
                if !self.sp.contains_key(&sp_str){
                    self.sp.insert(sp_str.clone(),Vec::new());
                }
                //self.sp.get_mut(&sp_str).unwrap().push(rule.clone());
                if let Some(mut rules) = self.sp.get_mut(&sp_str){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
            }
            //so
            if s.is_named_node() && p.is_blank_node() && o.is_named_node(){
                let so_str = format!("{}{}",s.to_string(),o.to_string());
                if !self.so.contains_key(&so_str){
                    self.so.insert(so_str.clone(),Vec::new());
                }
                //self.so.get_mut(&so_str).unwrap().push(rule.clone());
                if let Some(mut rules) = self.so.get_mut(&so_str){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
            }
            //po
            if s.is_blank_node() && p.is_named_node() && o.is_named_node(){
                let po_str = format!("{}{}",p.to_string(),o.to_string());
                if !self.po.contains_key(&po_str){
                    self.po.insert(po_str.clone(),Vec::new());
                }
                //self.po.get_mut(&po_str).unwrap().push(rule.clone());
                if let Some(mut rules) = self.po.get_mut(&po_str){
                    if !rules.contains(&rule) {rules.push(rule.clone())};
                }
            }
            //spo
            if s.is_blank_node() && p.is_blank_node() && o.is_blank_node() {
                //self.spo.push(rule.clone());
                if !self.spo.contains(&rule) {self.spo.push(rule.clone())};

            }

        }
    }

    pub fn find_match(&self, quad: &Quad)->Vec<&Rule>{
        let mut matched_triples: Vec<&Rule> = Vec::new();
        //check s
        if let Some(rule) = self.s.get(&quad.subject.to_string()){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check p
        if let Some(rule) = self.p.get(&quad.predicate.to_string()){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check o
        if let Some(rule) = self.o.get(&quad.predicate.to_string()){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check so
        if let Some(rule) = self.so.get(&format!("{}{}",quad.subject.to_string(),quad.object.to_string())){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check po
        if let Some(rule) = self.po.get(&format!("{}{}",quad.predicate.to_string(),quad.object.to_string())){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        //check sp
        if let Some(rule) = self.sp.get(&format!("{}{}",quad.subject.to_string(),quad.predicate.to_string())){
            rule.iter().for_each(|r|matched_triples.push(r));
        }
        self.spo.iter().for_each(|r| matched_triples.push(r));

        matched_triples
    }
}
#[cfg(test)]
mod tests {
    use oxigraph::model::{GraphName, NamedNode};
    use super::*;

    #[test]
    fn test_binding_retrieval() {
        let mut index = RuleIndex::new();
        let mut body = Vec::new();
        body.push(ReasonerTriple::new("?s".to_string(),"http://test/t".to_string(),"?o".to_string()));
        body.push(ReasonerTriple::new("?o".to_string(),"http://test/t".to_string(),"?q".to_string()));
        let rule = Rule{body:body, head:ReasonerTriple::new("?s".to_string(),"http://test/t".to_string(),"?q".to_string())};
        index.add(Rc::new(rule));
        let quad = Quad::new(NamedNode::new("http://example.com/s").unwrap(),
                             NamedNode::new("http://test/t").unwrap(),
                             NamedNode::new("http://example.com/o").unwrap(),
                             GraphName::DefaultGraph);
        let matches = index.find_match(&quad);
        assert_eq!(matches.len(), 1);
    }
}