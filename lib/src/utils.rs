use crate::{Encoder, Rule, Triple};

pub struct Utils;

impl Utils{
    pub fn decode_triple(encoder: &Encoder, triple:  &Triple) -> String {
        let s = encoder.decode(&triple.s.to_encoded()).unwrap();
        let p = encoder.decode(&triple.p.to_encoded()).unwrap();
        let o = encoder.decode(&triple.o.to_encoded()).unwrap();
        format!("{} {} {}",s,p,o)
    }
    pub fn decode_rule(encoder:&Encoder, rule: &Rule) -> String{
        let body = rule.body.iter().map(|t|Self::decode_triple(encoder,t) + ",").collect::<String>();
        let head = Self::decode_triple(encoder,&rule.head);
        format!("{{{}}}=>{{{}}}",body,head)
    }
}