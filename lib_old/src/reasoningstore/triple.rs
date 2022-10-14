use std::collections::HashMap;
use std::path::Component::Prefix;
use oxigraph::model::{BlankNode, NamedNode, NamedOrBlankNode};

fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}
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
#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct ReasonerTriple{
    pub s: NamedOrBlankNode,
    pub p: NamedOrBlankNode,
    pub o: NamedOrBlankNode
}
impl ReasonerTriple{
    pub fn new(s: String, p: String, o: String) -> ReasonerTriple{
        ReasonerTriple{s:ReasonerTriple::convert(s),p:ReasonerTriple::convert(p), o:ReasonerTriple::convert(o)}
    }
    pub fn new_from_prefixes(s: String, p: String, o: String, prefixes: &PrefixMapper) -> ReasonerTriple{
        let expand_s = prefixes.expand(s);
        let expand_p = prefixes.expand(p);
        let expand_o = prefixes.expand(o);
        ReasonerTriple{s:ReasonerTriple::convert(expand_s),p:ReasonerTriple::convert(expand_p), o:ReasonerTriple::convert(expand_o)}
    }
    fn convert(iri: String) -> NamedOrBlankNode{
        let result : NamedOrBlankNode;
        if iri.starts_with('?'){
            let var_name = &iri[1..];
            result = NamedOrBlankNode::from(BlankNode::new(var_name).unwrap());
        }else{
            let mut iri_prefix = iri;
            if iri_prefix.starts_with('<'){
                iri_prefix = rem_first_and_last(iri_prefix.as_str()).to_string();
            }
            result = NamedOrBlankNode::from(NamedNode::new(iri_prefix).unwrap());
        }
        result
    }
    pub fn to_string(&self) -> String{
        let mut final_string: String = "".to_owned();
        match &self.s{
            NamedOrBlankNode::NamedNode(node_iri) => final_string.push_str(&format!(" {} ", node_iri.to_string())),
            NamedOrBlankNode::BlankNode(var_name) => final_string.push_str(&format!(" ?{} ", var_name.as_str())),
        }
        match &self.p{
            NamedOrBlankNode::NamedNode(node_iri) => final_string.push_str(&format!(" {} ", node_iri.to_string())),
            NamedOrBlankNode::BlankNode(var_name) => final_string.push_str(&format!(" ?{} ", var_name.as_str())),
        }
        match &self.o{
            NamedOrBlankNode::NamedNode(node_iri) => final_string.push_str(&format!(" {} .", node_iri.to_string())),
            NamedOrBlankNode::BlankNode(var_name) => final_string.push_str(&format!(" ?{} .", var_name.as_str())),
        }
        final_string
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn prefix_mapper_test() {
        let mut mapper = PrefixMapper::new();
        mapper.add("test".to_string(),"http://test.org/test/".to_string());
        assert_eq!(mapper.expand("test:testing".to_string()), "http://test.org/test/testing");
    }
    #[test]
    fn prefix_mapper_default_test() {
        let mut mapper = PrefixMapper::new();
        mapper.add("".to_string(),"http://test.org/test/".to_string());
        assert_eq!(mapper.expand(":testing".to_string()), "http://test.org/test/testing");
    }
    #[test]
    fn no_prefix_test() {
        let mut mapper = PrefixMapper::new();
        mapper.add("".to_string(),"http://test.org/test/".to_string());
        assert_eq!(mapper.expand("http://test.org/test/".to_string()), "http://test.org/test/");
        assert_eq!(mapper.expand("?test".to_string()), "?test");

    }
}
