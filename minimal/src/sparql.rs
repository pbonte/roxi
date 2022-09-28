use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::empty;
use std::rc::Rc;
use spargebra::Query;
use spargebra::Query::Select;
use spargebra::algebra::*;
use spargebra::term::TriplePattern;
use crate::{Encoder, Parser, Syntax, TermImpl, Triple, TripleIndex, TripleStore, VarOrTerm};
use crate::sparql::EncodedTerm::NamedNode;
use crate::sparql::PlanNode::QuadPattern;
use crate::tripleindex::EncodedBinding;

fn extract_triples(triple_patterns: &Vec<TriplePattern>, encoder: &mut Encoder)-> Vec<Triple>{
    let mut triples = Vec::new();
    println!("BGP: {:?}", triple_patterns);
    for TriplePattern{subject: s , predicate: p,object:o } in triple_patterns{
        println!("subject: {:?}", s.to_string());
        println!("predicate: {:?}", p.to_string());
        println!("object: {:?}", o.to_string());
        triples.push(Triple::from(s.to_string(),p.to_string(),o.to_string(), encoder));
    }
    triples
}
#[derive(Debug)]
enum PlanExpression{
    Constant(TermImpl),
    Variable(usize),
    Greater(Box<Self>, Box<Self>),
    Done
}
#[derive(Debug)]
enum PlanNode{
    Join{left: Box<Self>, right: Box<Self>},
    QuadPattern{pattern: Triple},
    Project {
        child: Box<Self>,
        mapping: Vec<usize>,
    },
    Filter{
        child: Box<Self>,
        expression: Box<PlanExpression>,
    },
    Done
}
fn new_join(left: PlanNode, right: PlanNode) -> PlanNode{
    PlanNode::Join {left:Box::new(left),right: Box::new(right)}
}
fn extract_query_plan(graph_pattern: &GraphPattern, encoder: &mut Encoder) -> PlanNode {
    match graph_pattern {
        GraphPattern::Bgp {patterns}=> patterns.iter().map(|t| QuadPattern {pattern:Triple::from(t.subject.to_string(),t.predicate.to_string(),t.object.to_string(), encoder)}).
            reduce(new_join).unwrap(),
        GraphPattern::Project {inner,variables}=>{

            PlanNode::Project {child: Box::new(extract_query_plan(inner,encoder)), mapping: variables.iter().map(|v|encoder.add(v.as_str().to_string())).collect()}
        },
        GraphPattern::Filter {expr, inner} =>{
            println!("Expression: {:?}",expr);
            println!("inner: {:?}",inner);
            PlanNode::Filter{child: Box::new(extract_query_plan(inner, encoder)), expression: Box::new(extract_expression(expr, encoder))}
        }
        _ => PlanNode::Done,
    }
}

fn extract_expression(expression: &Expression,encoder: &mut Encoder) -> PlanExpression {
    match expression {
        Expression::Greater(a,b)=>{
            PlanExpression::Greater(Box::new(extract_expression(a, encoder)),Box::new(extract_expression(b, encoder)))
        },
        Expression::Variable(var)=>PlanExpression::Variable(encoder.add(var.as_str().to_string())),
        Expression::Literal(value)=>PlanExpression::Constant(TermImpl{iri:value.value().parse::<usize>().unwrap()}),
        _=> PlanExpression::Done

    }
}

fn evaluate_plan<'a>(plan_node: &'a PlanNode, triple_index: &'a TripleIndex) -> Box<dyn Iterator<Item=Vec<EncodedBinding>> + 'a> {
    match plan_node{
        PlanNode::QuadPattern {pattern: triple}=>{
            triple_index.query_help(&triple,None)
        },
        PlanNode::Project {child,mapping}=>{
            let child_it = evaluate_plan(child, triple_index);

            Box::new(child_it.map(|binding|{
                let projection : Vec<EncodedBinding>= binding.into_iter().filter(|b|mapping.contains(&b.var)).collect();
                projection
            }))
        },
        PlanNode::Join {left, right}=> {
            let left = evaluate_plan(left,triple_index);
            let right = evaluate_plan(right,triple_index);
            let mut left = left.peekable();
            let mut right = right.peekable();
            let left_peek = left.peek();
            let right_peek = right.peek();
            if let (Some(left_bindings),Some(right_bindings)) = (left_peek,right_peek){
                let left_vars: Vec<usize> = left_bindings.iter().map(|b|b.var).collect();
                let intersection: Vec<usize> = right_bindings.iter().filter(|b|left_vars.contains(&b.var)).map(|b|b.var).collect();
                //create the hash for left side
                let mut hash = HashMap::new();
                left.into_iter().for_each(|bindings|{
                    for binding in &bindings{
                        if binding.var == *intersection.get(0).unwrap(){
                            if ! hash.contains_key(&binding.val){
                                hash.insert(binding.val, Vec::new());
                            }
                            hash.get_mut(&binding.val).unwrap().push(bindings.clone());
                        }
                    }
                });
                return Box::new(right.map(move |bindings|{
                    let mut all_bindings = Vec::new();
                    for binding in &bindings{
                        if binding.var == *intersection.get(0).unwrap(){
                            if hash.contains_key(&binding.val){

                                for hashed in  hash.get(&binding.val).unwrap(){
                                    let mut new_bindings = bindings.clone();
                                    new_bindings.append(&mut hashed.clone());
                                    all_bindings.push(new_bindings);
                                }


                            }
                        }
                    }
                    all_bindings
                }).flatten());
            }
            Box::new(empty())},
        PlanNode::Filter {child, expression}=>{
            let child = evaluate_plan(child,triple_index);
            let expression = eval_expression(expression);
            Box::new(child.filter(move|bindings|{
                if let Some(EncodedTerm::BooleanLiteral(true)) = expression(bindings){
                    true
                }else{
                    false
                }
            }))
            },
        PlanNode::Done => Box::new(empty())
    }
}
fn eval_expression<'a>(expression: &'a PlanExpression) ->  Box<dyn Fn(&Vec<EncodedBinding>) -> Option<EncodedTerm> + 'a>{
    match expression{
        PlanExpression::Greater(a,b)=>{
            let a = eval_expression(a);
            let b = eval_expression(b);
            // Box::new(move |bindings| match a(bindings).and_then(|v| to_bool(&v)) {
            //     Some(true) => b(bindings),
            //     Some(false) => Some(false.into()),
            //     None => {
            //         if Some(false) == b(bindings).and_then(|v| to_bool(&v)) {
            //             Some(false.into())
            //         } else {
            //             None
            //         }
            //     }
            // })
                Box::new(move |bindings| {
                    let b_res = b(bindings);

                    let r = match a(bindings) {
                        Some(EncodedTerm::IntegerLiteral(int_val_a)) => match b_res{
                            Some(EncodedTerm::IntegerLiteral(int_val_b)) => int_val_a.partial_cmp(&int_val_b).into(),
                            _ => None
                        },
                        Some(EncodedTerm::StringLiteral(str_val_a)) => match b(bindings){
                            Some(EncodedTerm::StringLiteral(str_val_b))=> str_val_a.partial_cmp(&str_val_b),
                            _ => None
                        },
                        _ => None
                    };

                    Some((r.unwrap() == Ordering::Greater).into())
                        })
        },
        PlanExpression::Variable(v)=> Box::new(move |bindings|{
            let var_value : Vec<&EncodedBinding> = bindings.iter().filter(|b|b.var==*v).collect();
            var_value.get(0).iter().map(|v|v.val.into()).next()
            //Some(var_value.get(0).unwrap().val.into())
        }),
        PlanExpression::Constant(t) => {
            let t = t.clone();
            Box::new(move |_| Some(EncodedTerm::IntegerLiteral (t.iri.clone())))
        },
        _ => Box::new(|bindings|Some(false.into()))
    }

}
fn to_bool(term: &EncodedTerm) -> Option<bool> {
    match term {
        EncodedTerm::BooleanLiteral(value) => Some(*value),
        EncodedTerm::StringLiteral(value) => Some(!value.is_empty()),
        EncodedTerm::IntegerLiteral(value) => Some(*value != 0),
        _ => None,
    }
}
pub enum EncodedTerm{
    NamedNode {
        iri_id: usize,
    },
    StringLiteral(String),
    IntegerLiteral(usize),
    BooleanLiteral(bool)
}
impl From<bool> for EncodedTerm {
    fn from(value: bool) -> Self {
        Self::BooleanLiteral(value)
    }
}
impl From<String> for EncodedTerm {
    fn from(value: String) -> Self {
        Self::StringLiteral(value)
    }
}
impl From<usize> for EncodedTerm {
    fn from(value: usize) -> Self {
        Self::IntegerLiteral(value)
    }
}
#[test]
fn test_sparql_parser(){
    //load triples
    let nquads = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/> .
<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Student> <http://example.com/somethingelse> .
<http://example.com/foo> <http://test/hasVal> \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> <http://example.com/somethingelse> .";
    let mut encoder = Encoder::new();
    let triples = Parser::parse_triples(nquads, &mut encoder, Syntax::NQuads).unwrap();
    let mut index = TripleIndex::new();
    triples.into_iter().for_each(|t| index.add(t));

    let query_str = "Select * WHERE {  ?s <http://test/hasVal> ?o2  . Filter(?o2 > 1). }";
    let query = Query::parse(query_str, None).unwrap();
    println!("{}",query.to_sse());
    //let mut bgp = Vec::new();
    match query {
        spargebra::Query::Select {
            pattern, base_iri, ..
        } => {
            println!("Select query");
            let plan = extract_query_plan(&pattern,&mut encoder);
            println!("Query plan: {:?}", plan);
            let iterator = evaluate_plan(&plan,&index);
            for item in iterator{
                println!("item {:?}", item);
            }

            // match pattern {
            //     GraphPattern::Bgp {patterns}=> println!("BGP: {:?}", patterns),
            //     GraphPattern::Project {inner,variables}=>
            //         match *inner{
            //             GraphPattern::Bgp {patterns}=> {bgp.append(&mut extract_triples(&patterns, &mut encoder)); ()},
            //             _ => (),
            //         }
            //     _ => println!("Something else"),
            // }
        }
        spargebra::Query::Ask {
            pattern, base_iri, ..
        } => {
            println!("Ask query");
        }
        spargebra::Query::Construct {
            template,
            pattern,
            base_iri,
            ..
        } => {
            println!("Construct query");
        }
        spargebra::Query::Describe {
            pattern, base_iri, ..
        } => {
            println!("Describe query");
        }
    }
    println!("encoder {:?}", encoder);
    // println!("BGP: {:?}",bgp);
}