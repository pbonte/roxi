use std::cell::RefCell;
use std::rc::Rc;
use crate::csprite::CSprite;
use crate::{Parser, Triple};
use crate::time_window::{TimeWindow, TimeWindowConsumer};

pub struct WindowReasoner {
    pub store: CSprite,
    prev: Vec<(i32, Rc<Triple>)>
}
impl WindowReasoner {
    pub fn new() -> WindowReasoner{
        WindowReasoner{ store: CSprite::new(), prev: Vec::new()}
    }
    fn compute_diff(new : &Vec<(i32, Rc<Triple>)>, old: &Vec<(i32, Rc<Triple>)>) -> (Vec<(i32, Rc<Triple>)>,Vec<(i32, Rc<Triple>)>){
        if old.is_empty(){
            return (new.clone(),Vec::new());
        }
        if new.is_empty(){
            return (Vec::new(), old.clone());
        }
        let mut additions = Vec::new();
        let mut removals = Vec::new();
        //compute additions
        let mut found = false;
        for item in new.iter(){
            if let Some(old_item) = old.last(){
                //println!("Additions Old: {:?}, new {:?}", old_item, item);
                if item.eq(old_item){
                   found = true;
                }else if found{
                    additions.push(item.clone());
                }
            }
        }
        //compute removals
        for old_item in old.iter() {
            if let Some(new_item) = new.first(){
                //println!("Removals Old: {:?}, new {:?}", old_item, new_item);

                if !new_item.eq(old_item){
                    removals.push(old_item.clone());
                }else{
                    break;
                }
            }
        }

        (additions,removals)
    }
}

impl TimeWindowConsumer<Triple> for WindowReasoner {

    fn update(&mut self, data: Vec<(i32, Rc<Triple>)>, ts:&i32){
        println!("Received data: {:?},", data.len());
        let (new, old) = WindowReasoner::compute_diff(&data,&self.prev);
        //self.store.clear();
        self.store.window_update(new,old,ts);
        self.prev = data;
    }
}

#[test]
fn test_transitive(){

    let mut window = TimeWindow::new(4,2);

    let data ="{?a in ?b.?b in ?c}=>{?a in ?c}\n\
        :1 in :0.\n\
        :2 in :1.\n\
        :3 in :2.\n\
        :4 in :3.\n\
        :5 in :4.\n\
        :6 in :5";

    let mut reasoner = WindowReasoner::new();

    let (content, rules) = Parser::parse(data.to_string(),&mut reasoner.store.encoder);
    reasoner.store.add_rules(rules);
    let consumer = Rc::new(RefCell::new(reasoner));
    window.register_consumer(consumer.clone());


    content.into_iter().enumerate().for_each(|(i, t)| window.add(t, i as i32));

    //contains 4 triples and 1 inferred triple
    assert_eq!(10, consumer.borrow_mut().store.len());
}
#[test]
fn test_compute_diff(){
    let mut window = TimeWindow::new(4,2);

    let data ="{?a in ?b.?b in ?c}=>{?a in ?c}\n\
        :1 in :0.\n\
        :2 in :1.\n\
        :3 in :2.\n\
        :4 in :3.\n\
        :5 in :4.\n\
        :6 in :5.\n\
        :7 in :6.\n\
        :8 in :7.\n\
        :9 in :8";

    let mut reasoner = WindowReasoner::new();

    let (content, rules) = Parser::parse(data.to_string(),&mut reasoner.store.encoder);
    reasoner.store.add_rules(rules);
    println!("Encoding: {:?}", reasoner.store.encoder);
    let consumer = Rc::new(RefCell::new(reasoner));
    window.register_consumer(consumer.clone());


    content.into_iter().enumerate().for_each(|(i, t)| window.add(t, i as i32));

    //contains 4 triples and 1 inferred triple
    assert_eq!(10, consumer.borrow_mut().store.len());
}