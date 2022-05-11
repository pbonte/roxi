use std::collections::HashMap;
use std::rc::Rc;
use crate::Triple;

pub struct TripleIndex{
    pub triples: Vec<Triple>,
    spo:HashMap<usize,  HashMap<usize,Vec<usize>>>,
    pos:HashMap<usize,  HashMap<usize,Vec<usize>>>,
    osp:HashMap<usize,  HashMap<usize,Vec<usize>>>,
}

impl TripleIndex {

}


impl TripleIndex {
    pub fn len(&self) -> usize {
        self.triples.len()
    }
    pub fn get(&self, index: usize) ->Option<&Triple>{
        self.triples.get(index)
    }
    pub fn new() -> TripleIndex {
       TripleIndex{triples: Vec::new(), spo: HashMap::new(), pos: HashMap::new(), osp: HashMap::new()}
    }
    pub fn add(&mut self,  triple:Triple){

        if ! self.spo.contains_key(&triple.s.to_encoded()){
            self.spo.insert(triple.s.to_encoded(), HashMap::new());
        }
        if ! self.spo.get(&triple.s.to_encoded()).unwrap().contains_key(&triple.p.to_encoded()){
            self.spo.get_mut(&triple.s.to_encoded()).unwrap().insert(triple.p.to_encoded(), Vec::new());
        }
        self.spo.get_mut(&triple.s.to_encoded()).unwrap().get_mut(&triple.p.to_encoded()).unwrap().push(triple.o.to_encoded());
        //pos
        if ! self.pos.contains_key(&triple.p.to_encoded()){
            self.pos.insert(triple.p.to_encoded(), HashMap::new());
        }
        if ! self.pos.get(&triple.p.to_encoded()).unwrap().contains_key(&triple.o.to_encoded()){
            self.pos.get_mut(&triple.p.to_encoded()).unwrap().insert(triple.o.to_encoded(), Vec::new());
        }
        self.pos.get_mut(&triple.p.to_encoded()).unwrap().get_mut(&triple.o.to_encoded()).unwrap().push(triple.s.to_encoded());
        //osp
        if ! self.osp.contains_key(&triple.o.to_encoded()){
            self.osp.insert(triple.o.to_encoded(), HashMap::new());
        }
        if ! self.osp.get(&triple.o.to_encoded()).unwrap().contains_key(&triple.s.to_encoded()){
            self.osp.get_mut(&triple.o.to_encoded()).unwrap().insert(triple.s.to_encoded(), Vec::new());
        }
        self.osp.get_mut(&triple.o.to_encoded()).unwrap().get_mut(&triple.s.to_encoded()).unwrap().push(triple.p.to_encoded());
        self.triples.push(triple);
    }
    pub fn contains(&self, triple: &Triple)->bool{
        if ! self.osp.contains_key(&triple.o.to_encoded()){
            false
        }else{
            if ! self.osp.get(&triple.o.to_encoded()).unwrap().contains_key(&triple.s.to_encoded()){
                false
            }else{
                self.osp.get(&triple.o.to_encoded()).unwrap().get(&triple.s.to_encoded()).unwrap().contains(&triple.p.to_encoded())
            }
        }
    }

}