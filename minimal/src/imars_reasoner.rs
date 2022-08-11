use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::imars::{ImarsWindow, WindowConsumer};
use crate::{Encoder, Triple, TripleStore};


struct ImarsReasoner {
    store: TripleStore,
    new: Vec<(i32, Rc<Triple>)>,
    old: Vec<(i32, Rc<Triple>)>,
}
impl ImarsReasoner {
    pub fn new() -> ImarsReasoner{
        ImarsReasoner{ store: TripleStore::new(),  new: vec![], old: vec![] }
    }
}

impl WindowConsumer<Triple> for ImarsReasoner {

    fn update(&mut self, new: Vec<(i32, Rc<Triple>)>, old: Vec<(i32, Rc<Triple>)>, ts: i32) -> Vec<Triple>{
        println!("Received new: {:?}, old: {:?}", new.len(), old.len());
        new.into_iter().for_each(|(ts, triple)|self.store.add_ref(triple));
        let mat_triples = self.store.materialize();
        old.into_iter().for_each(|(ts,t)|self.store.remove_ref(t));
        mat_triples
    }
}

#[test]
fn test_integration(){
    let mut window = ImarsWindow::new(4, 2);

    let data="{?a a :C9}=>{?a a :C10}\n\
            {?a a :C4}=>{?a a :C10}\n\
            :a a :C0.\n\
            :b a :C1.\n\
            :c a :C2.\n\
            :d a :C3.\n\
            :e a :C4.\n\
            :f a :C5.\n\
            :g a :C6.\n\
            :i a :C7.\n\
            :j a :C8.\n\
            :k a :C9.";
    let mut reasoner = ImarsReasoner::new();

    let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut reasoner.store.encoder);
    reasoner.store.add_rules(rules);
    let mut consumer = Rc::new(RefCell::new(reasoner));
    window.register_consumer(consumer.clone());


    content.into_iter().enumerate().for_each(|(i, t)| window.add(t, i as i32));

    //contains 4 triples and 1 inferred triple
    assert_eq!(5, window.len());

}
#[test]
fn test_transitive(){
    let mut window = ImarsWindow::new(4, 2);

    let data="{?a in ?b.?b in ?c}=>{?a in ?c}\n\
            :1 in :0.\n\
            :2 in :1.\n\
            :3 in :2.\n\
            :4 in :3. \n\
            :5 in :4. \n\
            :6 in :5";
    let mut reasoner = ImarsReasoner::new();


    let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut reasoner.store.encoder);
    reasoner.store.add_rules(rules);
    let mut consumer = Rc::new(RefCell::new(reasoner));
    window.register_consumer(consumer.clone());


    content.into_iter().enumerate().for_each(|(i, t)| window.add(t, i as i32));

    //contains 4 triples and 1 inferred triple
    assert_eq!(19, window.len());
}