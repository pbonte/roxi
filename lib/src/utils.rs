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
    pub fn remove_literal_tags(literal: &str) -> String{
        if literal.contains("^^"){
            let mut split = literal.split("^^");
            if let Some(val) = split.next() {
               let new_str =  &val[1..val.len() - 1];
                new_str.to_string()
            }else{
                literal.to_string()
            }
        }else{
            literal.to_string()
        }
    }
}

mod test{
    use crate::utils::Utils;

    #[test]
    fn test_remove_literal_tages(){
        let literal = "\"10\"^^<http://www.w3.org/2001/XMLSchema#integer>";
        let expected = "10".to_string();
        assert_eq!(expected, Utils::remove_literal_tags(literal));
    }
}