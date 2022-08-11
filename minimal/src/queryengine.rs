use std::rc::Rc;
use crate::{Binding, Triple, TripleIndex};

pub trait QueryEngine{
    fn query(data: &TripleIndex, query_triples:&Vec::<Triple>,triple_counter : Option<usize>) -> Option<Binding>;
}
pub struct SimpleQueryEngine;


impl QueryEngine for SimpleQueryEngine {
    fn query(data: &TripleIndex, query_triples: &Vec::<Triple>, triple_counter: Option<usize>) -> Option<Binding> {
        let mut bindings = Binding::new();
        for query_triple in query_triples {
            //let current_bindings = self.query(query_triple,triple_counter);
            if let Some(current_bindings) = data.query(query_triple, triple_counter) {
                bindings = bindings.join(&current_bindings);
            } else {
                return None;
            }
        }
        Some(bindings)
    }
}
// pub fn query(&self, query_triple:&Triple, triple_counter : Option<usize>) -> Binding{
//     let mut bindings = Binding::new();
//     let mut counter = if let Some(size) = triple_counter{size} else {self.triple_index.len()};
//     for Triple{s,p,o} in self.triple_index.triples.iter().take(counter){
//         match &query_triple.s{
//             VarOrTerm::Var(s_var)=> bindings.add(&s_var.name,s.as_Term().iri),
//             VarOrTerm::Term(s_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (s_term,s.as_Term()) {
//                 if !iri.eq(iri2){break;}
//             }
//         }
//         match &query_triple.p{
//             VarOrTerm::Var(p_var)=> bindings.add(&p_var.name,p.as_Term().iri),
//             VarOrTerm::Term(p_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (p_term,p.as_Term()) {
//                 if !iri.eq(iri2){break;}
//             }
//         }
//         match &query_triple.o{
//             VarOrTerm::Var(o_var)=> bindings.add(&o_var.name,o.as_Term().iri),
//             VarOrTerm::Term(o_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (o_term,o.as_Term()) {
//                 if !iri.eq(iri2){break;}
//             }
//         }
//     }
//     bindings
// }