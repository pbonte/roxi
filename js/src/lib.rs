extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use roxi::parser::Syntax;
use roxi::TripleStore;


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
pub struct RoxiReasoner{
    reasoner: TripleStore
}
#[wasm_bindgen]
impl RoxiReasoner{
    pub fn new() -> RoxiReasoner{
        RoxiReasoner{reasoner: TripleStore::new()}
    }
    pub fn add_abox(&mut self, abox:String){
        self.reasoner.load_triples(abox.as_ref(), Syntax::NTriples);
    }
    pub fn add_rules(&mut self, rules:String){
        self.reasoner.load_rules(rules.as_str());
    }
    pub fn len_abox(&self)->usize{
        self.reasoner.len()
    }
    pub fn materialize(&mut self){
        self.reasoner.materialize();
    }
    pub fn get_abox_dump(& self)->String{
        self.reasoner.content_to_string()
    }
}