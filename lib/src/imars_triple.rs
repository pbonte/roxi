use std::cell::RefCell;
use deepmesa::lists::LinkedList;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use deepmesa::lists::linkedlist::Node;
use crate::csprite::CSprite;
use crate::Triple;


/// A generic Windowing operator that implements IMaRs functionality.
///
/// Each window time-based window and has a width and sliding parameter to define its size.
/// The window assigner does not duplicate the items in the window across multiple windows but maintains
/// the state of a single window, adding and remove based on the timestamps.
/// # Examples
/// ```
/// use lib::imars::ImarsWindow;
/// let mut window :ImarsWindow<i32> = ImarsWindow::new(2,2);
/// window.add(100,0);
/// window.add(101,1);
/// window.add(102,2);
/// window.add(103,3);
/// assert_eq!(2,window.len());
/// ```
/// IMaRs allows to update values with newer timestamps (used for reasoning):
///
/// ```
/// use lib::imars::ImarsWindow;
/// let mut window :ImarsWindow<i32> = ImarsWindow::new(4,2);
/// window.add(100,0);
/// window.add(101,1);
/// window.add(102,2);
/// window.add(103,3);
/// assert_eq!(4,window.len());
/// window.add(100,4);
/// assert_eq!(4,window.len());
/// ```
/// Consumers can be added to consume the data when the window triggers, i.e. when the temporal bounds
/// of the window are reached:
/// ```
/// use std::cell::RefCell;
/// use std::rc::Rc;
/// use lib::imars::{ImarsWindow, SimpleWindowConsumer};
///
/// let mut window :ImarsWindow<i32> = ImarsWindow::new(2,2);
/// let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
/// window.register_consumer(consumer.clone());
/// ```

pub struct ImarsWindowTriple {
     content: LinkedList<(i32, Rc<Triple>)>,
     consumers: Vec<Rc<CSprite>>,
     width: i32,
     slide: i32,
     time: i32,
     pending_adds: Vec<(i32,Rc<Triple>)>,
     index: HashMap<Rc<Triple>,Node<(i32,Rc<Triple>)>>
}

impl ImarsWindowTriple{
    /// Creates a new time-based window with a certain width and slide
    pub fn new(width: i32, slide: i32) -> ImarsWindowTriple{
        ImarsWindowTriple{content: LinkedList::new(), consumers: Vec::new(), width, slide, time: 0, pending_adds: Vec::new(), index: HashMap::new()}
    }
    /// Adds an item to the window and updates its content, this can either be:
    /// - Add the item to the window and to nothing when the new timestamp does not exceed the bounds of the current window
    /// - Add the item and update the window, i.e. remove old items that have expired based on their timestamp
    /// - The item is already in the window but has an updated timestamp, this will update the current item
    pub fn add(&mut self, item:Triple, ts:i32) {

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
            let consumers = self.consumers.clone();
            consumers.iter().for_each(|mut item|{
                    let mut reference = item;
                    let updates = reference.update(self.pending_adds.clone(), old_values.clone(), ts);
                    updates.into_iter().for_each(|t| self.add(t,ts));
                });

            self.pending_adds.clear();
        }
    }
    fn update(&mut self, item:Rc<Triple>, ts:i32){
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
    /// Returns the length of the content of the window
    pub fn len(&self) -> usize{
        self.content.len()
    }
    fn add_to_list_and_index(&mut self, item:Rc<Triple>, ts:i32){
        let node_ref = self.content.push_tail((ts,item.clone()));
        //add to index
        self.index.insert(item,node_ref);
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
    fn progress_time_and_delete_old(&mut self, ts: &i32) -> Vec<(i32,Rc<Triple>)>{
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
    /// Adds consumer that can be notified with updates
    pub fn register_consumer(&mut self, consumer: Rc<CSprite>) {
        self.consumers.push(consumer);
    }
}






