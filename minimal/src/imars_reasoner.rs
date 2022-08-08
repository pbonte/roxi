use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use crate::imars::{ImarsWindow, WindowConsumer};
use crate::{Encoder, Triple, TripleStore};


struct ImarsReasoner {
    store: TripleStore,
    windows: Vec<Box<ImarsWindow<Triple>>>,
    new: Vec<(i32, Rc<Triple>)>,
    old: Vec<(i32, Rc<Triple>)>,
}
impl ImarsReasoner {
    pub fn new() -> ImarsReasoner{
        ImarsReasoner{ store: TripleStore::new(), windows: Vec::new(), new: vec![], old: vec![] }
    }
}

impl WindowConsumer<Triple> for ImarsReasoner {

    fn update(&mut self, new: Vec<(i32, Rc<Triple>)>, old: Vec<(i32, Rc<Triple>)>, ts: i32) {
        println!("Received new: {:?}, old: {:?}", new.len(), old.len());
        new.into_iter().for_each(|(ts, triple)|self.store.add_ref(triple));
        //self.store.add_ref()
        let mat_triples = self.store.materialize();
        println!("Materialized: {:?}", mat_triples);
        for window in self.windows.iter_mut(){
            mat_triples.into_iter().for_each(|t| window.add(t, ts));
            break;
        }
    }
}

#[test]
fn test_integration(){

    let mut encoder = Encoder::new();
    let data="{?a a :C0}=>{?a a :C1}\n\
            :a a :C0.\n\
            :b a :C1.\n\
            :c a :C2.\n\
            :d a :C3.\n\
            :e a :C4.\n\
            :f a :C5.\n\
            :g a :C6.";
    let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);
    let mut reasoner = ImarsReasoner::new();
    reasoner.store.add_rules(rules);
    let mut consumer = Rc::new(RefCell::new(reasoner));
    let mut window = ImarsWindow::new(4, 2);
    window.register_consumer(consumer.clone());
    content.into_iter().enumerate().for_each(|(i, t)|window.add(t,i as i32));
    //contains 4 triples and 1 inferred triple
    assert_eq!(5,window.len());

}