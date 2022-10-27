use crate::{Encoder, Rule, Triple, VarOrTerm};
use rio_turtle::{NTriplesParser, TurtleError,TurtleParser, TriGParser, NQuadsParser};
use rio_api::parser::{QuadsParser, TriplesParser};
use rio_api::model::NamedNode;

mod n3rule_parser;

pub struct Parser;
#[derive(PartialEq)]
pub enum Syntax {NTriples, Turtle, TriG, NQuads}

impl Default for Syntax{
    fn default() -> Self {
        Syntax::NTriples
    }
}

impl Parser {
    pub fn parse_triples(data: &str, encoder: &mut Encoder, syntax: Syntax) -> Result<Vec<Triple>, &'static str>{
        if syntax == Syntax::Turtle || syntax == Syntax::NTriples {
            Self::parse_triples_helper(data,encoder,syntax)
        }else{
            Self::parse_quads_helper(data,encoder, syntax)
        }

    }
    fn parse_quads_helper(data: &str, encoder: &mut Encoder, syntax: Syntax) -> Result<Vec<Triple>, &'static str> {
        let mut triples = Vec::new();
        let closure_quad = &mut |t: rio_api::model::Quad| {
            let s = VarOrTerm::new_term(t.subject.to_string(), encoder);
            let p = VarOrTerm::new_term(t.predicate.to_string(), encoder);
            let o = VarOrTerm::new_term(t.object.to_string(), encoder);
            let g = t.graph_name.map(|g|VarOrTerm::new_term(g.to_string(),encoder));
            triples.push(Triple { s, p, o, g });
            Ok(()) as Result<(), TurtleError>
        };

        let result = match syntax {
            Syntax::TriG =>  TriGParser::new(data.as_ref(), None).parse_all(closure_quad),
            Syntax::NQuads =>  NQuadsParser::new(data.as_ref()).parse_all(closure_quad),
            _ => NQuadsParser::new(data.as_ref()).parse_all(closure_quad)
        };
        match result {
            Ok(_) =>Ok(triples),
            _ => Err("Parsing error!")
        }


    }
    fn parse_triples_helper(data: &str, encoder: &mut Encoder, syntax: Syntax) -> Result<Vec<Triple>, &'static str>{
        let mut triples = Vec::new();
        let closure_triple = &mut |t: rio_api::model::Triple| {
            let s = VarOrTerm::new_term(t.subject.to_string(),encoder);
            let p = VarOrTerm::new_term(t.predicate.to_string(),encoder);
            let o = VarOrTerm::new_term(t.object.to_string(), encoder);
            triples.push(Triple { s, p, o, g: None });
            Ok(()) as Result<(), TurtleError>
        };
        let result = match syntax {
            Syntax::NTriples =>  NTriplesParser::new(data.as_ref()).parse_all(closure_triple).unwrap(),
            Syntax::Turtle =>  TurtleParser::new(data.as_ref(), None).parse_all(closure_triple).unwrap(),
            _=> NTriplesParser::new(data.as_ref()).parse_all(closure_triple).unwrap(),
        };
        match result {
            () =>Ok(triples),
            _ => Err("Parsing error!")
        }
    }
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
        Triple { s, p, o, g: None }
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
    pub fn parse_rules(parse_string: &str, encoder: &mut Encoder) -> Result<Vec<Rule>,&'static str>{
        n3rule_parser::parse(parse_string,encoder)
    }
}

mod test{
    use super::*;
    #[test]
    fn test_parsing(){
        let ntriples_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let mut encoder = Encoder::new();
        let triples = Parser::parse_triples(ntriples_file, &mut encoder, Syntax::NTriples).unwrap();
        assert_eq!(4, triples.len());

        let trig_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>, <http://schema.org/Student> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>; <http://schema.org/name> \"Bar\" .";
        let mut encoder = Encoder::new();
        let triples = Parser::parse_triples(trig_file, &mut encoder, Syntax::TriG).unwrap();
        assert_eq!(5, triples.len());

        let turtle = "@prefix schema: <http://schema.org/> .
<http://example.com/foo> a schema:Person ;
    schema:name  \"Foo\" .
<http://example.com/bar> a schema:Person ;
    schema:name  \"Bar\" .";
        let mut encoder = Encoder::new();
        let triples = Parser::parse_triples(turtle, &mut encoder, Syntax::Turtle).unwrap();
        assert_eq!(4, triples.len());

        let nquads = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" <http://example.com/> .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let mut encoder = Encoder::new();
        let triples = Parser::parse_triples(nquads                  , &mut encoder, Syntax::NQuads).unwrap();
        assert_eq!(4, triples.len());

        let parsing_error = "<http://example.com/foo> http://www.w3.org/1999/02/22-rdf-syntax-ns#typehema.org/Person";
        let mut encoder = Encoder::new();
        let triples = Parser::parse_triples(parsing_error                  , &mut encoder, Syntax::NQuads);
        assert_eq!(true,triples.is_err());

    }
    #[test]
    fn test_empty_abox_parsing(){
        let ntriples_file = "";
        let mut encoder = Encoder::new();
        let triples = Parser::parse_triples(ntriples_file, &mut encoder, Syntax::NTriples).unwrap();
        assert_eq!(0, triples.len());
    }

}