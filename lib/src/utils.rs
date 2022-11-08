use crate::{Encoder, Rule, Triple};

pub struct Utils;

impl Utils{
    pub fn decode_triple( triple:  &Triple) -> String {
        let s = Encoder::decode(&triple.s.to_encoded()).unwrap();
        let p = Encoder::decode(&triple.p.to_encoded()).unwrap();
        let o = Encoder::decode(&triple.o.to_encoded()).unwrap();
        format!("{} {} {}",s,p,o)
    }
    pub fn decode_rule(rule: &Rule) -> String{
        let body = rule.body.iter().map(|t|Self::decode_triple(t) + ",").collect::<String>();
        let head = Self::decode_triple(&rule.head);
        format!("{{{}}}=>{{{}}}",body,head)
    }
}