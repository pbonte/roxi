use std::cell::RefCell;
use deepmesa::lists::LinkedList;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use deepmesa::lists::linkedlist::Node;

/// A consumer for retrieving new and expired data from the window
pub trait TimeWindowConsumer<T>{
    fn update(&mut self, data: Vec<(i32, Rc<T>)>, ts:&i32);
}


pub struct  SimpleWindowConsumer<T>{
    windows: Vec<Box<TimeWindow<T>>>,
    data: Vec<(i32, Rc<T>)>

}
impl <T> SimpleWindowConsumer<T>{
    pub fn new() -> SimpleWindowConsumer<T>{
        SimpleWindowConsumer{windows: Vec::new(), data: vec![] }
    }
}
impl <T> TimeWindowConsumer<T> for SimpleWindowConsumer<T> {

    fn update(&mut self, data: Vec<(i32, Rc<T>)>,_ts:&i32) {
        //println!("Received new: {:?}, old: {:?}", new.len(), old.len());
        self.data = data;
    }
}

pub struct TimeWindow<T> {
     content: LinkedList<(i32, Rc<T>)>,
     consumers: Vec<Rc<RefCell<dyn TimeWindowConsumer<T>>>>,
     width: i32,
     slide: i32,
     time: i32,
     pending_adds: Vec<(i32,Rc<T>)>,
}


impl<T: Clone> TimeWindow< T> where T: Eq + Hash{
    /// Creates a new time-based window with a certain width and slide
    pub fn new(width: i32, slide: i32) -> TimeWindow< T>{
        TimeWindow {content: LinkedList::new(), consumers: Vec::new(), width, slide, time: 0, pending_adds: Vec::new()}
    }
    /// Adds an item to the window and updates its content, this can either be:
    /// - Add the item to the window and to nothing when the new timestamp does not exceed the bounds of the current window
    /// - Add the item and update the window, i.e. remove old items that have expired based on their timestamp
    /// - The item is already in the window but has an updated timestamp, this will update the current item
    pub fn add(&mut self, item:T, ts:i32) {

        let rc_item = Rc::new(item.clone());

        //add the item
        self.content.push_tail((ts,rc_item.clone()));

        if self.does_window_trigger(ts){
            self.update_window_open_time(ts);
            let window_content = self.progress_time_and_extract_content(&ts);

            let consumers = self.consumers.clone();
            let last_valid_ts = self.get_last_valid_time_for(&ts);
            consumers.iter().for_each(|mut item|{
                    let mut reference = item.borrow_mut();
                    reference.update(window_content.clone(),&last_valid_ts);
                });

            self.pending_adds.clear();
        }
    }

    /// Returns the length of the content of the window
    pub fn len(&self) -> usize{
        self.content.len()
    }

    fn get_last_valid_time_for(&self, new_time: &i32) -> i32{
        cmp::max(0,*new_time - self.width)
    }
    fn does_window_trigger(&mut self, ts: i32) -> bool {
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
    fn progress_time_and_extract_content(&mut self, ts: &i32) -> Vec<(i32, Rc<T>)>{ let mut window_content = Vec::new();
        while let Some((timestamp, item)) = self.content.front(){
            if *timestamp<= self.get_last_valid_time_for(ts){
                self.content.pop_front();
            }else{
                break;
            }
        }
        self.content.iter().for_each(|(ts,item)| window_content.push((*ts,item.clone())));
        window_content
    }

    /// Adds consumer that can be notified with updates
    pub fn register_consumer(&mut self, consumer: Rc<RefCell<dyn TimeWindowConsumer<T>>>) {
        self.consumers.push(consumer);
    }
}



#[test]
fn test_new_window(){
   let window : TimeWindow<i32> = TimeWindow::new(5, 2);
   assert_eq!(window.len(),0);
}

#[test]
fn test_add_to_window(){
    let mut window : TimeWindow<i32> = TimeWindow::new(5, 2);
    window.add(100,0);
    assert_eq!(window.content.front(),Some(&(0,Rc::from(100))));
}

#[test]
fn test_window_shift(){
    let mut window : TimeWindow<i32> = TimeWindow::new(2, 2);
    window.add(100, 0);
    window.add(101, 1);
    window.add(102, 2);
    window.add(103, 3);
    window.progress_time_and_extract_content(&3);
    assert_eq!(window.content.front(),Some(&(2,Rc::from(102))));
}
#[test]
fn test_window_bound_calculation(){
    let mut window : TimeWindow<i32> = TimeWindow::new(3, 2);
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

    let mut window : TimeWindow<i32> = TimeWindow::new(5, 3);
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
    let mut window : TimeWindow<i32> = TimeWindow::new(2, 2);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    assert_eq!(0,consumer.borrow_mut().data.len());
    window.add(100,0);
    window.add(101,1);
    window.add(102,2);
    window.add(103,3);

    assert_eq!(2,consumer.borrow_mut().data.len());
}
#[test]
fn test_delete(){
    let mut window : TimeWindow<i32> = TimeWindow::new(2, 2);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    assert_eq!(0,consumer.borrow_mut().data.len());
    window.add(100,0);
    window.add(101,1);
    window.add(102,2);
    window.add(103,3);
    assert_eq!(2,window.content.len());

    assert_eq!(2,consumer.borrow_mut().data.len());
}

#[test]
fn test_update(){
    let mut window : TimeWindow<i32> = TimeWindow::new(4, 2);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    assert_eq!(0,consumer.borrow_mut().data.len());
    window.add(100,0);
    window.add(101,1);
    window.add(102,2);
    window.add(103,3);
    assert_eq!(4,window.content.len());
    window.add(100,4);
    window.add(100,5);

    assert_eq!(4,window.content.len());
}





