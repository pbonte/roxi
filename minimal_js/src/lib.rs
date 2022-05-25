extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use minimal::*;
use minimal::ruleindex::RuleIndex;
use minimal::tripleindex::TripleIndex;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub struct  RustReasoner{
    reasoner: TripleStore
}
#[wasm_bindgen]
impl RustReasoner{
    pub fn from(data:String) -> RustReasoner{
        let mut encoder = Encoder::new();
        let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);


        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add(rule);
        }
        let mut store = TripleStore{rules:Vec::new(), rules_index , triple_index, encoder };
        RustReasoner{reasoner: store}
    }

    pub fn materialize(&mut self){
        self.reasoner.materialize();
    }
    pub fn content_to_string(&mut self) -> String{
        self.reasoner.content_to_string()
    }
    pub fn len(&self)->usize{
        self.reasoner.len()
    }
}
