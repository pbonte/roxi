use crate::{Encoder, Rule, Triple, VarOrTerm};

pub struct Parser;

impl Parser {
    fn parse_triple(data: &str, encoder: &mut Encoder) -> Triple {
        let items: Vec<&str> = data.split(" ").collect();
        let s = items.get(0).unwrap();
        let p = items.get(1).unwrap();

        let o = if items.get(2).unwrap().ends_with(".") {
            let mut o_chars = items.get(2).unwrap().chars();
            o_chars.next_back();
            o_chars.as_str()
        } else {
            items.get(2).unwrap()
        };
        let mut convert_item = |item: &&str| { if item.starts_with("?") { VarOrTerm::new_var(item.to_string(), encoder) } else { VarOrTerm::new_term(item.to_string(), encoder) } };
        let s = convert_item(s);
        let p = convert_item(p);
        let o = convert_item(&o);
        Triple { s, p, o }
    }
    fn rem_first_and_last(value: &str) -> &str {
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        chars.as_str()
    }
    pub fn parse(data: String, encoder: &mut Encoder) -> (Vec<Triple>, Vec<Rule>) {
        let mut rules = Vec::new();
        let mut content = Vec::new();
        //line by line
        for line in data.split("\n") {
            if line.contains("=>") {
                //process rule
                let rule: Vec<&str> = line.split("=>").collect();
                let body = Self::rem_first_and_last(rule.get(0).unwrap());
                let head = Self::rem_first_and_last(rule.get(1).unwrap());
                let head_triple = Self::parse_triple(head, encoder);
                let mut body_triples = Vec::new();
                for body_triple in body.split(".") {
                    if body_triple.len() > 0 {
                        body_triples.push(Self::parse_triple(body_triple, encoder));
                    }
                }
                rules.push(Rule { head: head_triple, body: body_triples })
            } else {
                //process triple
                if line.len() > 0 {
                    let triple = Self::parse_triple(line, encoder);
                    content.push(triple);
                }
            }
        }
        (content, rules)
    }
}