use std::borrow::Borrow;
use std::cell::RefCell;
use deepmesa::lists::LinkedList;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use deepmesa::lists::linkedlist::Node;

pub struct Triple{
    s:  String,
    p: String,
    o: String
}

pub trait WindowConsumer<T>{
    fn update(&mut self, new: Vec<(i32,Rc<T>)>, old: Vec<(i32,Rc<T>)>);
}
pub struct  SimpleWindowConsumer<T>{
    windows: Vec<Box<ImarsWindow<T>>>,
    new: Vec<(i32, Rc<T>)>,
    old: Vec<(i32, Rc<T>)>
}
impl <T> SimpleWindowConsumer<T>{
    pub fn new() -> SimpleWindowConsumer<T>{
        SimpleWindowConsumer{windows: Vec::new(), new: vec![], old: vec![] }
    }
}
impl <T> WindowConsumer<T> for SimpleWindowConsumer<T> {

    fn update(&mut self, new: Vec<(i32, Rc<T>)>, old: Vec<(i32, Rc<T>)>) {
        //println!("Received new: {:?}, old: {:?}", new.len(), old.len());
        self.new = new;
        self.old = old;
    }
}
pub struct ImarsWindow<T> {
     content: LinkedList<(i32, Rc<T>)>,
     consumers: Vec<Rc<RefCell<dyn WindowConsumer<T>>>>,
     width: i32,
     slide: i32,
     time: i32,
     pending_adds: Vec<(i32,Rc<T>)>,
     index: HashMap<Rc<T>,Node<(i32,Rc<T>)>>
}

impl<T: Clone> ImarsWindow< T> where T: Eq + Hash{
    pub fn new(width: i32, slide: i32) -> ImarsWindow< T>{
        ImarsWindow{content: LinkedList::new(), consumers: Vec::new(), width, slide, time: 0, pending_adds: Vec::new(), index: HashMap::new()}
    }
    pub fn add(&mut self, item:T, ts:i32) {

        let rc_item = Rc::new(item.clone());
        //check if item is already present
        if self.index.contains_key(&rc_item) {
            //update the item
            self.update(rc_item,ts);
        } else {
            //add the item
            self.pending_adds.push((ts, rc_item.clone()));
            self.add_to_list_and_index(rc_item, ts);
        }
        if self.does_window_trigger(ts){
            self.update_window_open_time(ts);
            let old_values = self.progress_time_and_delete_old(&ts);

            self.consumers.iter().for_each(|mut item|
                {
                    let mut reference = item.borrow_mut();
                    reference.update(self.pending_adds.clone(), old_values.clone());
                }
            );
            self.pending_adds.clear();
        }
    }
    pub fn update(&mut self, item:Rc<T>, ts:i32){
        if let Some(node) = self.index.get(&item){
            // cut node from middle
            if let Some(content) = self.content.pop_node(&node){
                //add it to end with updated timestamp
                let updated_node_ref = self.content.push_tail((ts,content.1));
                //update the index
                self.index.insert(item,updated_node_ref);
            }

        }
    }
    fn add_to_list_and_index(&mut self, item:Rc<T>, ts:i32){
        let node_ref = self.content.push_tail((ts,item.clone()));
        //add to index
        self.index.insert(item,node_ref);
    }
    fn get_last_valid_time_for(&self, new_time: &i32) -> i32{
        cmp::max(0,*new_time - self.width)

    }
    pub fn does_window_trigger(&mut self, ts: i32) -> bool {
        if ts > self.time + self.width {
            true
        }else{
            false
        }
    }
    fn update_window_open_time(&mut self, ts: i32){
        let mut residue = (ts - self.width)/self.slide;
        if (ts - self.width) % self.slide != 0{
            residue +=1;
        }
        self.time =  residue * self.slide;
    }
    fn progress_time_and_delete_old(&mut self, ts: &i32) -> Vec<(i32,Rc<T>)>{
        let mut old_values = Vec::new();
        let mut peek = self.content.front();
        while let Some((timestamp, item)) = peek{
            if *timestamp<= self.get_last_valid_time_for(ts){
                if let Some(old_val) = self.content.pop_front(){
                    //remove from index
                    self.index.remove(&old_val.1);
                    old_values.push(old_val);
                }
                peek = self.content.front();

            }else{
                break;
            }
        }
        old_values
    }
    pub fn register_consumer(&mut self, consumer: Rc<RefCell<dyn WindowConsumer<T>>>) {
        self.consumers.push(consumer);
    }
}



#[test]
fn test_new_window(){
   let mut window :ImarsWindow<i32> = ImarsWindow::new(5,2);
   assert_eq!(window.content.len(),0);
}

#[test]
fn test_add_to_window(){
    let mut window :ImarsWindow<i32> = ImarsWindow::new(5,2);
    window.add(100,0);
    assert_eq!(window.content.front(),Some(&(0,Rc::from(100))));
}

#[test]
fn test_window_shift(){
    let mut window :ImarsWindow<i32> = ImarsWindow::new(2,2);
    window.add_to_list_and_index(Rc::from(100), 0);
    window.add_to_list_and_index(Rc::from(101), 1);
    window.add_to_list_and_index(Rc::from(102), 2);
    window.add_to_list_and_index(Rc::from(103), 3);
    window.progress_time_and_delete_old(&3);
    assert_eq!(window.content.front(),Some(&(2,Rc::from(102))));
}
#[test]
fn test_window_bound_calculation(){
    let mut window :ImarsWindow<i32> = ImarsWindow::new(3,2);
    assert_eq!(false,window.does_window_trigger(2));
    assert_eq!(false,window.does_window_trigger(3));
    assert_eq!(true,window.does_window_trigger(4));
    assert_eq!(true,window.does_window_trigger(5));
    window.update_window_open_time(5);
    assert_eq!(false,window.does_window_trigger(5));
    assert_eq!(true,window.does_window_trigger(6));
    assert_eq!(true,window.does_window_trigger(7));
    window.update_window_open_time(8);
    assert_eq!(false,window.does_window_trigger(9));
    assert_eq!(true,window.does_window_trigger(10));

    let mut window :ImarsWindow<i32> = ImarsWindow::new(5,3);
    assert_eq!(false,window.does_window_trigger(2));
    assert_eq!(true,window.does_window_trigger(6));
    window.update_window_open_time(6);
    assert_eq!(false,window.does_window_trigger(7));
    assert_eq!(true,window.does_window_trigger(9));
    window.update_window_open_time(10);
    assert_eq!(false,window.does_window_trigger(11));
    assert_eq!(true,window.does_window_trigger(12));
}

#[test]
fn test_consumer(){
    let mut window :ImarsWindow<i32> = ImarsWindow::new(2,2);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    assert_eq!(0,consumer.borrow_mut().new.len());
    window.add(100,0);
    window.add(101,1);
    window.add(102,2);
    window.add(103,3);

    assert_eq!(4,consumer.borrow_mut().new.len());
    assert_eq!(2,consumer.borrow_mut().old.len());
}
#[test]
fn test_delete(){
    let mut window :ImarsWindow<i32> = ImarsWindow::new(2,2);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    assert_eq!(0,consumer.borrow_mut().new.len());
    window.add(100,0);
    window.add(101,1);
    window.add(102,2);
    window.add(103,3);
    assert_eq!(2,window.content.len());
    assert_eq!(2,window.index.len());

    assert_eq!(4,consumer.borrow_mut().new.len());
    assert_eq!(2,consumer.borrow_mut().old.len());
}

#[test]
fn test_update(){
    let mut window :ImarsWindow<i32> = ImarsWindow::new(4,2);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    assert_eq!(0,consumer.borrow_mut().new.len());
    window.add(100,0);
    window.add(101,1);
    window.add(102,2);
    window.add(103,3);
    assert_eq!(4,window.content.len());
    assert_eq!(4,window.index.len());
    window.add(100,4);
    assert_eq!(4,window.content.len());
    assert_eq!(4,window.index.len());
}
#[test]
fn test_throughput(){
    let mut window :ImarsWindow<i32> = ImarsWindow::new(1000,10);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    assert_eq!(0,consumer.borrow_mut().new.len());
    use std::time::Instant;
    let now = Instant::now();
    for i in 1..1000{
        window.add(i,i);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
#[test]
fn test_imars_throughput(){
    let mut list = LinkedList::<i32>::with_capacity(10);
    list.push_tail(1);
    let middle = list.push_tail(100);
    list.push_tail(2);
    for i in 1..1000000{
        list.push_back(i);
    }

// get the value of the node in the middle of the list in O(1)
// time.
    assert_eq!(list.node(&middle), Some(&100));
// remove the middle node in O(1) time
    list.pop_node(&middle);
// once the middle node is removed, the handle is invalid
    assert_eq!(list.node(&middle), None);

}


