use std::collections::HashMap;
use std::rc::Rc;
use crate::{Binding, Encoder, Parser, TermImpl, Triple, TripleStore};

pub struct TripleIndex{
    pub triples: Vec<Triple>,
    spo:HashMap<usize,  HashMap<usize,Vec<(usize,usize)>>>,
    pos:HashMap<usize,  HashMap<usize,Vec<(usize,usize)>>>,
    osp:HashMap<usize,  HashMap<usize,Vec<(usize,usize)>>>,
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
    pub fn remove_ref(&mut  self, triple: Rc<Triple>) {
        //remove spo
        if self.spo.contains_key(&triple.s.to_encoded()) &&
            self.spo.get(&triple.s.to_encoded()).unwrap().contains_key(&triple.p.to_encoded()){
            let spo_values = self.spo.get_mut(&triple.s.to_encoded()).unwrap().get_mut(&triple.p.to_encoded()).unwrap();
            spo_values.retain(|(val,counter)| *val != triple.o.to_encoded());
        }
        //remove pos
        if self.pos.contains_key(&triple.p.to_encoded()) &&
            self.pos.get(&triple.p.to_encoded()).unwrap().contains_key(&triple.o.to_encoded()){
            let values = self.pos.get_mut(&triple.p.to_encoded()).unwrap().get_mut(&triple.o.to_encoded()).unwrap();
            values.retain(|(val,counter)| *val != triple.s.to_encoded());
        }
        // remove osp
        if self.osp.contains_key(&triple.o.to_encoded()) &&
            self.osp.get(&triple.o.to_encoded()).unwrap().contains_key(&triple.s.to_encoded()){
            let values = self.osp.get_mut(&triple.o.to_encoded()).unwrap().get_mut(&triple.s.to_encoded()).unwrap();
            values.retain(|(val,counter)| *val != triple.p.to_encoded());
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
        self.spo.get_mut(&triple.s.to_encoded()).unwrap().get_mut(&triple.p.to_encoded()).unwrap().push((triple.o.to_encoded(),self.counter));
        //pos
        if ! self.pos.contains_key(&triple.p.to_encoded()){
            self.pos.insert(triple.p.to_encoded(), HashMap::new());
        }
        if ! self.pos.get(&triple.p.to_encoded()).unwrap().contains_key(&triple.o.to_encoded()){
            self.pos.get_mut(&triple.p.to_encoded()).unwrap().insert(triple.o.to_encoded(), Vec::new());
        }
        self.pos.get_mut(&triple.p.to_encoded()).unwrap().get_mut(&triple.o.to_encoded()).unwrap().push((triple.s.to_encoded(),self.counter));
        //osp
        if ! self.osp.contains_key(&triple.o.to_encoded()){
            self.osp.insert(triple.o.to_encoded(), HashMap::new());
        }
        if ! self.osp.get(&triple.o.to_encoded()).unwrap().contains_key(&triple.s.to_encoded()){
            self.osp.get_mut(&triple.o.to_encoded()).unwrap().insert(triple.s.to_encoded(), Vec::new());
        }
        self.osp.get_mut(&triple.o.to_encoded()).unwrap().get_mut(&triple.s.to_encoded()).unwrap().push((triple.p.to_encoded(),self.counter));
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
                for (encoded, counter) in self.osp.get(&triple.o.to_encoded()).unwrap().get(&triple.s.to_encoded()).unwrap(){
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
                    for (encoded_match,counter) in indexes2.iter(){
                        if *counter<=counter_check {
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
                    for (encoded_match,counter) in indexes2.iter(){
                        if *counter<=counter_check {
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
                    for (encoded_match,counter) in indexes2.iter(){
                        if *counter<=counter_check {
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
                    for (encoded_match,counter) in p_values.iter(){
                        if *counter<=counter_check {
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
                    for (encoded_match,counter) in o_values.iter(){
                        if *counter<=counter_check {
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
                    for (encoded_match,counter) in s_values.iter(){
                        if *counter<=counter_check {
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
                    for (encoded_match,counter) in o_values.iter(){
                        if *counter<=counter_check {
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
                    for (encoded_match,counter) in indexes2.iter(){
                        if *counter<=counter_check {
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
    pub fn clear(&mut self){
        self.triples.clear();
        self.spo.clear();
        self.osp.clear();
        self.pos.clear();
        self.counter = 0;
    }

}

#[test]
fn test_remove(){
    let mut index = TripleIndex::new();
    let mut encoder = Encoder::new();
    let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClass.";
    let (content, _rules) = Parser::parse(data.to_string(),&mut encoder);
    let  rc_triples: Vec<Rc<Triple>> = content.into_iter().map(|t|Rc::new(t)).collect();
    rc_triples.iter().for_each(|t|index.add_ref(t.clone()));
    assert_eq!(4, index.len());
    index.remove_ref(rc_triples.get(0).unwrap().clone());
    assert_eq!(3, index.len());
}

#[test]
fn test_query_fact(){
    let mut index = TripleIndex::new();
    let mut encoder = Encoder::new();
    let data=":a a :C.\n\
                :b a :D.\n\
                {:a a :C}=>{:a a :D}";
    let (content, rules) = Parser::parse(data.to_string(),&mut encoder);
    content.into_iter().for_each(|t|index.add(t));
    let query = rules.get(0).unwrap().body.get(0).unwrap();
    let result = index.query(query,None);
    assert_eq!(true, result.is_some());

}