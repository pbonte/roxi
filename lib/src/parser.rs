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
    pub fn parse_triples(data: &str,  syntax: Syntax) -> Result<Vec<Triple>, String>{
        if syntax == Syntax::Turtle || syntax == Syntax::NTriples {
            Self::parse_triples_helper(data,syntax)
        }else{
            Self::parse_quads_helper(data, syntax)
        }

    }
    fn parse_quads_helper(data: &str,  syntax: Syntax) -> Result<Vec<Triple>, String> {
        let mut triples = Vec::new();
        let closure_quad = &mut |t: rio_api::model::Quad| {
            let s = VarOrTerm::new_term(t.subject.to_string());
            let p = VarOrTerm::new_term(t.predicate.to_string());
            let o = VarOrTerm::new_term(t.object.to_string());
            let g = t.graph_name.map(|g|VarOrTerm::new_term(g.to_string()));
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
            Err(parsing_error) => Err(format!("Parsing error! {:?}", parsing_error.to_string()))
        }


    }
    fn parse_triples_helper(data: &str,  syntax: Syntax) -> Result<Vec<Triple>, String>{
        let mut triples = Vec::new();
        let closure_triple = &mut |t: rio_api::model::Triple| {
            let s = VarOrTerm::new_term(t.subject.to_string());
            let p = VarOrTerm::new_term(t.predicate.to_string());
            let o = VarOrTerm::new_term(t.object.to_string());
            triples.push(Triple { s, p, o, g: None });
            Ok(()) as Result<(), TurtleError>
        };
        let result = match syntax {
            Syntax::NTriples =>  NTriplesParser::new(data.as_ref()).parse_all(closure_triple),
            Syntax::Turtle =>  TurtleParser::new(data.as_ref(), None).parse_all(closure_triple),
            _=> NTriplesParser::new(data.as_ref()).parse_all(closure_triple),
        };
        match result {
            Ok(_) =>Ok(triples),
            Err(parsing_error) => Err(format!("Parsing error! {:?}", parsing_error.to_string()))
        }
    }
    fn parse_triple(data: &str) -> Triple {
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
        let mut convert_item = |item: &&str| { if item.starts_with("?") { VarOrTerm::new_var(item.to_string()) } else { VarOrTerm::new_term(item.to_string()) } };
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
    pub fn parse(data: String) -> (Vec<Triple>, Vec<Rule>) {
        let mut rules = Vec::new();
        let mut content = Vec::new();
        //line by line
        for line in data.split("\n") {
            if line.contains("=>") {
                //process rule
                let rule: Vec<&str> = line.split("=>").collect();
                let body = Self::rem_first_and_last(rule.get(0).unwrap());
                let head = Self::rem_first_and_last(rule.get(1).unwrap());
                let head_triple = Self::parse_triple(head);
                let mut body_triples = Vec::new();
                for body_triple in body.split(".") {
                    if body_triple.trim().len() > 0 {
                        body_triples.push(Self::parse_triple(body_triple.trim()));
                    }
                }
                rules.push(Rule { head: head_triple, body: body_triples })
            } else {
                //process triple
                if line.len() > 0 {
                    let triple = Self::parse_triple(line);
                    content.push(triple);
                }
            }
        }
        (content, rules)
    }
    pub fn parse_rules(parse_string: &str) -> Result<Vec<Rule>,&'static str>{
        n3rule_parser::parse(parse_string)
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
        let triples = Parser::parse_triples(ntriples_file,  Syntax::NTriples).unwrap();
        assert_eq!(4, triples.len());

        let trig_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>, <http://schema.org/Student> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>; <http://schema.org/name> \"Bar\" .";
        let triples = Parser::parse_triples(trig_file,  Syntax::TriG).unwrap();
        assert_eq!(5, triples.len());

        let turtle = "@prefix schema: <http://schema.org/> .
<http://example.com/foo> a schema:Person ;
    schema:name  \"Foo\" .
<http://example.com/bar> a schema:Person ;
    schema:name  \"Bar\" .";
        let triples = Parser::parse_triples(turtle,  Syntax::Turtle).unwrap();
        assert_eq!(4, triples.len());

        let nquads = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" <http://example.com/> .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let triples = Parser::parse_triples(nquads,  Syntax::NQuads).unwrap();
        assert_eq!(4, triples.len());

        let parsing_error = "<http://example.com/foo> http://www.w3.org/1999/02/22-rdf-syntax-ns#typehema.org/Person";
        let triples = Parser::parse_triples(parsing_error ,  Syntax::NQuads);
        assert_eq!(true,triples.is_err());

    }
    #[test]
    fn test_empty_abox_parsing(){
        let ntriples_file = "";
        let triples = Parser::parse_triples(ntriples_file,Syntax::NTriples).unwrap();
        assert_eq!(0, triples.len());
    }

    #[test]
    fn test_error_abox_parsing(){
        let ntriples_file = "asdfadsf";
        match Parser::parse_triples(ntriples_file,Syntax::NTriples){
            Ok(result)=>assert_eq!(0, 1),
            Err(err)=>assert_eq!(0, 0)
        }

    }
    #[test]
    fn test_syntactic_sugar_rdf_type(){
        let ntriples_file = "<http://example2.com/a> a <http://www.test.be/test#SubClass> .";
        match Parser::parse_triples(ntriples_file,Syntax::Turtle){
            Ok(result)=>assert_eq!(1, result.len()),
            Err(err)=>assert_eq!(0, 1)
        }

    }
    #[test]
    fn test_white_space_in_rules(){
        let rules = "{?source a test:Source. }=>{?source a test:NeededInput.}";
        match Parser::parse_rules(rules){
            Ok(result)=>assert_eq!(1, result.len()),
            Err(err)=>assert_eq!(0, 1)
        }

    }

}