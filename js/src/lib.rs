extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use std::fmt::format;
use std::sync::{Arc, Mutex};
use std::thread;
use cfg_if::cfg_if;
use js_sys::{Array, Function};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use roxi::parser::Syntax;
use roxi::TripleStore;
use roxi::rsp::{OperationMode, ResultConsumer, RSPBuilder, RSPEngine, SimpleR2R};
use roxi::rsp::r2s::StreamOperator;
use roxi::rsp::s2r::{ReportStrategy, Tick, WindowTriple};
use roxi::sparql::Binding;

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
    pub fn query(&self, query: String)->Array {
        self.reasoner.query(query.as_str()).into_iter().map(|row|
            {
                let js_bindings : Vec<JsValue> = row.into_iter().map(|b|JSBinding{var: b.var, val: b.val}.into()).collect();
                let js_array = JsValue::from(js_bindings.into_iter().collect::<Array>());
                js_array
            }
        ).collect::<Array>()
    }
}

#[wasm_bindgen]
pub struct JSRSPEngine{
    engine: RSPEngine<WindowTriple, Vec<Binding>>
}
struct Test{
    f: js_sys::Function
}
unsafe impl Send for Test {}

#[wasm_bindgen]
#[derive(Debug)]
pub struct JSBinding{
    val:String, var:String
}
#[wasm_bindgen]
impl JSBinding{
    pub fn getValue(&self) -> String{
        self.val.clone()
    }
    pub fn getVar(&self) -> String {
        self.var.clone()
    }
    pub fn toString(&self) -> String{
        format!("Binding{{{:?}: {:?}}}",self.var.clone(), self.val.clone())
    }
}
#[wasm_bindgen]
impl JSRSPEngine{
    pub fn new(width: usize, slide: usize, rules: String, abox: String, query: String, f: &js_sys::Function) -> JSRSPEngine {
        let t = Arc::new(Mutex::new(Test{f: f.clone()}));
        let t2 = t.clone();
        let function: Box<dyn Fn(Vec<Binding>)-> () + Send + Sync> = Box::new(move |r|{
            //for x in r{
                let this = JsValue::null();
                //convert to JSBindings and JSValues
                let r_js: Vec<JsValue> = r.into_iter().map(|binding|JSBinding{var: binding.var, val: binding.val}.into()).collect();
                // convert to JS Array
                let x = JsValue::from(r_js.into_iter().collect::<Array>());
                let f = t2.lock().unwrap();
                let _ = f.f.call1(&this, &x);
            //}
            ();
        });

        let result_consumer : ResultConsumer<Vec<Binding>> = ResultConsumer{function: Arc::new(function)};
        let r2r = Box::new(SimpleR2R{item: TripleStore::new()});
        let mut engine = RSPBuilder::new(width,slide)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(&abox)
            .add_syntax(Syntax::NTriples)
            .add_rules(&rules)
            .add_query(&query)
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .set_operation_mode(OperationMode::SingleThread)
            .build();
        JSRSPEngine{engine}

    }
    pub fn add(&mut self, triple_string: String, ts: usize){
        let mut triple_string = triple_string.clone();
        if triple_string.ends_with("."){
            triple_string = triple_string[..triple_string.len() - 1].to_string();
        }
        let mut triple_string = triple_string.split(" ");

        let triple = WindowTriple{s:triple_string.next().unwrap().trim().to_string(),
            p:triple_string.next().unwrap().trim().to_string(),
            o: triple_string.next().unwrap().trim().to_string(),};

        self.engine.add(triple,ts);
    }
}