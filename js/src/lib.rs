extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use roxi::reasoningstore::ReasoningStore;

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
    reasoner: ReasoningStore
}
#[wasm_bindgen]
impl RoxiReasoner{
    pub fn new() -> RoxiReasoner{
        RoxiReasoner{reasoner: ReasoningStore::new()}
    }
    pub fn add_abox(&self, abox:String){
        self.reasoner.load_abox(abox.as_ref());
    }
    pub fn add_rules(&mut self, rules:String){
        self.reasoner.parse_and_add_rule(rules.as_str());
    }
    pub fn len_abox(&self)->usize{
        self.reasoner.len_abox()
    }
    pub fn len_rules(&self)->usize{
        self.reasoner.len_rules()
    }
    pub fn materialize(&mut self){
        self.reasoner.materialize();
    }
    pub fn get_abox_dump(&self)->String{
        self.reasoner.dump_as_string()
    }
}