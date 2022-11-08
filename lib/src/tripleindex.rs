use std::collections::HashMap;
use std::iter;
use std::iter::empty;
use std::rc::Rc;
use crate::{Binding, Encoder, Parser, TermImpl, Triple, TripleStore, VarOrTerm};
use either::*;

pub struct TripleIndex{
    pub triples: Vec<Triple>,
    spo:HashMap<usize,  HashMap<usize,Vec<(usize,usize, Option<TermImpl>)>>>,
    pos:HashMap<usize,  HashMap<usize,Vec<(usize,usize, Option<TermImpl>)>>>,
    osp:HashMap<usize,  HashMap<usize,Vec<(usize,usize, Option<TermImpl>)>>>,
    counter:usize,
}


impl TripleIndex {
    pub fn len(&self) -> usize {
        self.triples.len()
    }
    pub fn get(&self, index: usize) ->Option<&Triple>{
        self.triples.get(index)
    }
    pub fn new() -> TripleIndex {
       TripleIndex{triples: Vec::new(), spo: HashMap::new(), pos: HashMap::new(), osp: HashMap::new(),counter:0}
    }
    //todo fix references!
    pub(crate) fn add_ref(&mut self, triple: Rc<Triple>) {
        self.add(triple.as_ref().clone());
    }
    pub fn remove_ref(&mut  self, triple: &Triple) {
        //remove spo
        if self.spo.contains_key(&triple.s.to_encoded()) &&
            self.spo.get(&triple.s.to_encoded()).unwrap().contains_key(&triple.p.to_encoded()){
            let spo_values = self.spo.get_mut(&triple.s.to_encoded()).unwrap().get_mut(&triple.p.to_encoded()).unwrap();
            spo_values.retain(|(val,counter,_)| *val != triple.o.to_encoded());
        }
        //remove pos
        if self.pos.contains_key(&triple.p.to_encoded()) &&
            self.pos.get(&triple.p.to_encoded()).unwrap().contains_key(&triple.o.to_encoded()){
            let values = self.pos.get_mut(&triple.p.to_encoded()).unwrap().get_mut(&triple.o.to_encoded()).unwrap();
            values.retain(|(val,counter, _)| *val != triple.s.to_encoded());
        }
        // remove osp
        if self.osp.contains_key(&triple.o.to_encoded()) &&
            self.osp.get(&triple.o.to_encoded()).unwrap().contains_key(&triple.s.to_encoded()){
            let values = self.osp.get_mut(&triple.o.to_encoded()).unwrap().get_mut(&triple.s.to_encoded()).unwrap();
            values.retain(|(val,counter, _)| *val != triple.p.to_encoded());
        }
        self.triples.retain(|t| *t != *triple);
        self.counter-=1;
    }
    pub fn add(&mut self,  triple:Triple){

        if ! self.spo.contains_key(&triple.s.to_encoded()){
            self.spo.insert(triple.s.to_encoded(), HashMap::new());
        }
        if ! self.spo.get(&triple.s.to_encoded()).unwrap().contains_key(&triple.p.to_encoded()){
            self.spo.get_mut(&triple.s.to_encoded()).unwrap().insert(triple.p.to_encoded(), Vec::new());
        }
        self.spo.get_mut(&triple.s.to_encoded()).unwrap().get_mut(&triple.p.to_encoded()).unwrap().push((triple.o.to_encoded(),self.counter,triple.g.clone().map(|g|g.as_term().clone())));
        //pos
        if ! self.pos.contains_key(&triple.p.to_encoded()){
            self.pos.insert(triple.p.to_encoded(), HashMap::new());
        }
        if ! self.pos.get(&triple.p.to_encoded()).unwrap().contains_key(&triple.o.to_encoded()){
            self.pos.get_mut(&triple.p.to_encoded()).unwrap().insert(triple.o.to_encoded(), Vec::new());
        }
        self.pos.get_mut(&triple.p.to_encoded()).unwrap().get_mut(&triple.o.to_encoded()).unwrap().push((triple.s.to_encoded(),self.counter, triple.g.clone().map(|g|g.as_term().clone())));
        //osp
        if ! self.osp.contains_key(&triple.o.to_encoded()){
            self.osp.insert(triple.o.to_encoded(), HashMap::new());
        }
        if ! self.osp.get(&triple.o.to_encoded()).unwrap().contains_key(&triple.s.to_encoded()){
            self.osp.get_mut(&triple.o.to_encoded()).unwrap().insert(triple.s.to_encoded(), Vec::new());
        }
        self.osp.get_mut(&triple.o.to_encoded()).unwrap().get_mut(&triple.s.to_encoded()).unwrap().push((triple.p.to_encoded(),self.counter, triple.g.clone().map(|g|g.as_term().clone())));
        self.triples.push(triple);
        self.counter+=1;
    }
    pub fn contains(&self, triple: &Triple)->bool{
        if ! self.osp.contains_key(&triple.o.to_encoded()){
            false
        }else{
            if ! self.osp.get(&triple.o.to_encoded()).unwrap().contains_key(&triple.s.to_encoded()){
                false
            }else{
                for (encoded, counter, _) in self.osp.get(&triple.o.to_encoded()).unwrap().get(&triple.s.to_encoded()).unwrap(){
                    if encoded == &triple.p.to_encoded(){
                        return true;
                    }
                }
                return false;

            }
        }
    }
    pub fn query(&self, query_triple: &Triple,triple_counter : Option<usize>)->Option<Binding>{
        let mut matched_binding = Binding::new();
        let counter_check = if let Some(size) = triple_counter{size} else {self.counter};
        //?s p o
        if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()){
                if let Some(indexes2) = indexes.get(&query_triple.o.to_encoded()){
                    for (encoded_match,counter, graph_name) in indexes2.iter(){
                        if *counter<=counter_check {
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            matched_binding.add(&query_triple.s.to_encoded(),encoded_match.clone() );
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //s ?p o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()){
                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()){
                    for (encoded_match,counter, graph_name) in indexes2.iter(){
                        if *counter<=counter_check{
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            matched_binding.add(&query_triple.p.to_encoded(),encoded_match.clone() );
                        }else{
                            break;
                        }
                    }                }
            }
        }
        //s p ?o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()){
                if let Some(indexes2) = indexes.get(&query_triple.p.to_encoded()){
                    for (encoded_match,counter, graph_name) in indexes2.iter(){
                        if *counter<=counter_check {
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            matched_binding.add(&query_triple.o.to_encoded(),encoded_match.clone() );
                        }else{
                            break;
                        }
                    }                }
            }
        }
        //?s ?p o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()){
                for (s_key, p_values) in indexes.iter(){
                    for (encoded_match,counter, graph_name) in p_values.iter(){
                        if *counter<=counter_check {
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            matched_binding.add(&query_triple.s.to_encoded(),s_key.clone() );
                            matched_binding.add(&query_triple.p.to_encoded(),encoded_match.clone() );
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //s ?p ?o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()){
                for (p_key, o_values) in indexes.iter(){
                    for (encoded_match,counter, graph_name) in o_values.iter(){
                        if *counter<=counter_check {
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            matched_binding.add(&query_triple.p.to_encoded(),p_key.clone() );
                            matched_binding.add(&query_triple.o.to_encoded(),encoded_match.clone() );
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //?s p ?o
        else if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()){
                for (o_key, s_values) in indexes.iter(){
                    for (encoded_match,counter, graph_name) in s_values.iter(){
                        if *counter<=counter_check {
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            matched_binding.add(&query_triple.o.to_encoded(),o_key.clone() );
                            matched_binding.add(&query_triple.s.to_encoded(),encoded_match.clone() );
                        }else{
                            break;
                        }
                    }

                }
            }
        }
        //?s ?p ?o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_var() {
            for (s_key, p_index) in  self.spo.iter(){
                for (p_key, o_values) in p_index.iter(){
                    for (encoded_match,counter, graph_name) in o_values.iter(){
                        if *counter<=counter_check {
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            matched_binding.add(&query_triple.s.to_encoded(),s_key.clone() );
                            matched_binding.add(&query_triple.p.to_encoded(),p_key.clone() );
                            matched_binding.add(&query_triple.o.to_encoded(),encoded_match.clone() );
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //s p o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()){
                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()){
                    for (encoded_match,counter, graph_name) in indexes2.iter(){
                        if *counter<=counter_check {
                            if !Self::check_quad_match_and_add(&query_triple, &mut matched_binding, graph_name){
                                break;
                            }
                            if *encoded_match == query_triple.p.to_encoded() {
                                // return when triple has been found in knowlege base
                                return Some(matched_binding);
                            }
                        }else{
                            break;
                        }
                    }
                }
            }
        }

        if matched_binding.len() > 0{
            Some(matched_binding)
        }else{
            None
        }
    }



    pub fn query_help<'a>( &'a self, query_triple: &'a Triple,triple_counter : Option<usize>)->Box<dyn Iterator<Item=Vec<EncodedBinding>> + 'a>{

        //?s p o
        if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()){
                if let Some(indexes2) = indexes.get(&query_triple.o.to_encoded()){
                    Self::extract_binding_values_single_var(&query_triple.s, &query_triple.g, indexes2)
                }else{
                    Box::new(empty())
                }
            }else{
                Box::new(empty())
            }
        }
        //s ?p o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_term() {

            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()){
                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()){
                    Self::extract_binding_values_single_var(&query_triple.p, &query_triple.g, indexes2)
                    }else{
                    Box::new(empty())
                }
            }else{
                Box::new(empty())
            }
        }
        // //s p ?o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()){
                if let Some(indexes2) = indexes.get(&query_triple.p.to_encoded()){
                    Self::extract_binding_values_single_var(&query_triple.o, &query_triple.g, indexes2)
                }else{
                    Box::new(empty())
                }
            }else{
                Box::new(empty())
            }
        }
        // //?s ?p o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()) {
                Box::new(indexes.iter().map(|(s_key, p_values)| p_values.iter().zip(iter::repeat(s_key).take(p_values.len()))).flatten()
                    .map(|((encoded_match, counter, graph_name), s_key)|
                        {
                            let mut bindings = Vec::with_capacity(3);
                            bindings.push(EncodedBinding { var: query_triple.s.to_encoded().clone(), val: s_key.clone() });
                            bindings.push(EncodedBinding { var: query_triple.p.to_encoded().clone(), val: encoded_match.clone() });

                            match &query_triple.g {
                                Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                    bindings.push(EncodedBinding { var: var_name.name.clone(), val: graph_name.clone().unwrap().iri });
                                }
                                Some(VarOrTerm::Term(term)) if !graph_name.clone().map_or(false, |t| t.eq(term)) => {
                                    return None
                                }
                                _ => {}
                            }
                            Some(bindings)
                        }
                    ).flatten())
            }else{
                Box::new(empty())
            }
        }
        //s ?p ?o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()){
                Box::new( indexes.iter().map(|(key, values)| values.iter().zip(iter::repeat(key).take(values.len()))).flatten()
                    .map(|((encoded_match, counter, graph_name), key)|
                        {
                            let mut bindings = Vec::with_capacity(3);
                            bindings.push(EncodedBinding { var: query_triple.p.to_encoded().clone(), val: key.clone() });
                            bindings.push(EncodedBinding { var: query_triple.o.to_encoded().clone(), val: encoded_match.clone() });

                            match &query_triple.g {
                                Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                    bindings.push(EncodedBinding { var: var_name.name.clone(), val: graph_name.clone().unwrap().iri });
                                }
                                Some(VarOrTerm::Term(term)) if !graph_name.clone().map_or(false, |t| t.eq(term)) => {
                                    return None
                                }
                                _ => {}
                            }
                            Some(bindings)
                        }
                    ).flatten())

            }else{Box::new(empty())}
        }
        //?s p ?o
        else if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()){
                Box::new( indexes.iter().map(|(key, values)| values.iter().zip(iter::repeat(key).take(values.len()))).flatten()
                    .map(|((encoded_match, counter, graph_name), key)|
                        {
                            let mut bindings = Vec::with_capacity(3);
                            bindings.push(EncodedBinding { var: query_triple.s.to_encoded().clone(), val: encoded_match.clone() });
                            bindings.push(EncodedBinding { var: query_triple.o.to_encoded().clone(), val: key.clone() });

                            match &query_triple.g {
                                Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                    bindings.push(EncodedBinding { var: var_name.name.clone(), val: graph_name.clone().unwrap().iri });
                                }
                                Some(VarOrTerm::Term(term)) if !graph_name.clone().map_or(false, |t| t.eq(term)) => {
                                    return None
                                }
                                _ => {}
                            }
                            Some(bindings)
                        }
                    ).flatten())

            }else{Box::new(empty())}
        }
        // //?s ?p ?o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_var() {

            Box::new(self.spo.iter().map(|(s_key,p_vals)|p_vals.iter().zip(iter::repeat(s_key).take(p_vals.len()))).flatten()
                .map(|((p_key, o_values),s_key)|o_values.iter().zip(iter::repeat(p_key).take(o_values.len())).zip(iter::repeat(s_key).take(o_values.len()))).flatten()
                .map(|(((encoded_match,counter, graph_name),p_key),s_key)|
                    {
                        let mut bindings = Vec::with_capacity(3);
                        bindings.push(EncodedBinding { var: query_triple.s.to_encoded().clone(), val: s_key.clone() });
                        bindings.push(EncodedBinding { var: query_triple.p.to_encoded().clone(), val: p_key.clone() });
                        bindings.push(EncodedBinding { var: query_triple.o.to_encoded().clone(), val: encoded_match.clone() });

                        match &query_triple.g {
                            Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                bindings.push(EncodedBinding { var: var_name.name.clone(), val: graph_name.clone().unwrap().iri });
                            }
                            Some(VarOrTerm::Term(term)) if !graph_name.clone().map_or(false, |t| t.eq(term)) => {
                                return None
                            }
                            _ => {}
                        }
                        Some(bindings)
                    }
                ).flatten())
        }
        // //s p o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()){


                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()){
                   Box::new( indexes2.iter().map(|(encoded_match,counter, graph_name)|{
                        if *encoded_match == query_triple.p.to_encoded() {
                            // return when triple has been found in knowlege base
                            Some(Vec::with_capacity(0))
                        }else{
                            None
                        }

                    }).flatten())

                }else{
                    Box::new(empty())
                }
            }else{
                Box::new(empty())
            }
        }else{
            Box::new(empty())
        }
        //
        // if matched_binding.len() > 0{
        //     Some(matched_binding)
        // }else{
        //     None
        // }
    }

    fn extract_binding_values_single_var<'a> (variable: &'a VarOrTerm, graph_var: &'a Option<VarOrTerm>, indexes2: &'a Vec<(usize, usize, Option<TermImpl>)>) -> Box<dyn Iterator<Item=Vec<EncodedBinding>>+ 'a> {
        Box::new(indexes2.iter().map(move |(encoded_match, counter, graph_name)| {
            let mut bindings = Vec::with_capacity(2);
            bindings.push(EncodedBinding { var: variable.to_encoded().clone(), val: encoded_match.clone() });
            match graph_var {
                Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                    bindings.push(EncodedBinding { var: var_name.name.clone(), val: graph_name.clone().unwrap().iri });
                }
                Some(VarOrTerm::Term(term)) if !graph_name.clone().map_or(false, |t| t.eq(term)) => {
                    return None
                }
                _ => {}
            }
            Some(bindings)
        }).flatten())
    }
    fn check_quad_match_and_add(query_triple: &&Triple, matched_binding: &mut Binding, graph_name: &Option<TermImpl>) -> bool{
        match &query_triple.g {
            Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                matched_binding.add(&var_name.name, graph_name.clone().unwrap().iri);
                return true;
            }
            Some(VarOrTerm::Term(term)) if !graph_name.clone().map_or(false, |t| t.eq(term)) => {
                return  false;
            }
            _ => {
                return true;
            }
        }
    }
    pub fn clear(&mut self){
        self.triples.clear();
        self.spo.clear();
        self.osp.clear();
        self.pos.clear();
        self.counter = 0;
    }

}
#[derive(Debug,Clone)]
pub struct EncodedBinding{pub var: usize, pub val: usize}
pub struct QuadIterator <'a>{
    query: Triple,
    index: &'a TripleIndex
}
impl <'a> Iterator for QuadIterator<'a> {
    type Item = Binding;
    fn next(&mut self) -> Option<Self::Item>{
        None
    }
}
#[cfg(test)]
mod tests {
    use crate::Syntax;
    use super::*;
    #[test]
    fn test_remove() {
        let mut index = TripleIndex::new();
        let data = "<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClass.";
        let (content, _rules) = Parser::parse(data.to_string());
        let rc_triples: Vec<Rc<Triple>> = content.into_iter().map(|t| Rc::new(t)).collect();
        rc_triples.iter().for_each(|t| index.add_ref(t.clone()));
        assert_eq!(4, index.len());
        index.remove_ref(&rc_triples.get(0).unwrap().clone());
        assert_eq!(3, index.len());
    }

    #[test]
    fn test_query_fact() {
        let mut index = TripleIndex::new();
        let data = ":a a :C.\n\
                :b a :D.\n\
                {:a a :C}=>{:a a :D}";
        let (content, rules) = Parser::parse(data.to_string());
        content.into_iter().for_each(|t| index.add(t));
        let query = rules.get(0).unwrap().body.get(0).unwrap();
        let result = index.query(query, None);
        assert_eq!(true, result.is_some());
    }
    #[test]
    fn test_quad_filter(){
        let nquads = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" <http://example.com/> .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let triples = Parser::parse_triples(nquads,  Syntax::NQuads).unwrap();
        let mut index = TripleIndex::new();
        triples.into_iter().for_each(|t| index.add(t));
        let query_triple = Triple::from_with_graph_name("?s".to_string(),"?p".to_string(),"?o".to_string(),"http://example.com/".to_string());
        let bindings = index.query(&query_triple,None);
        assert_eq!(2, bindings.unwrap().len());
    }

    #[test]
    fn test_quad_query(){
        let nquads = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/> .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/somethingelse> .";
        let triples = Parser::parse_triples(nquads,  Syntax::NQuads).unwrap();
        let mut index = TripleIndex::new();
        triples.into_iter().for_each(|t| index.add(t));
        let query_triple = Triple::from_with_graph_name("http://example.com/foo".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://schema.org/Person".to_string(),"?g".to_string());
        let bindings = index.query(&query_triple,None).unwrap();
        assert_eq!(1, bindings.len());
        assert_eq!(&Encoder::add("<http://example.com/>".to_string()),
                   bindings.get(&Encoder::add("g".to_string())).unwrap().get(0).unwrap());
    }
    #[test]
    #[ignore]
    fn test_same_triple_in_multiple_graphs_query(){
        let nquads = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/> .
<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/somethingelse> .";
        let triples = Parser::parse_triples(nquads,  Syntax::NQuads).unwrap();
        let mut index = TripleIndex::new();
        triples.into_iter().for_each(|t| index.add(t));
        let query_triple = Triple::from_with_graph_name("http://example.com/foo".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://schema.org/Person".to_string(),"?g".to_string());
        let bindings = index.query(&query_triple,None).unwrap();
        assert_eq!(2, bindings.len());
        assert_eq!(&Encoder::add("<http://example.com/>".to_string()),
                   bindings.get(&Encoder::add("g".to_string())).unwrap().get(0).unwrap());
    }


    #[test]
    fn test_iterator(){
        let nquads = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> <http://example.com/> .
<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Student> <http://example.com/somethingelse> .";
        let triples = Parser::parse_triples(nquads,  Syntax::NQuads).unwrap();
        let mut index = TripleIndex::new();
        triples.into_iter().for_each(|t| index.add(t));
        let query_triple = Triple::from("?s".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"?o".to_string());
        let it = index.query_help(&query_triple,None);
        for item in it{
            println!("item {:?}", item);
        }

    }
}