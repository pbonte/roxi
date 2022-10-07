use std::collections::{HashMap, HashSet};
use std::{f64, mem};
use crate::Triple;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::Receiver;
#[cfg(not(test))]
use log::{info, warn, trace, debug}; // Use log crate when building application
#[cfg(test)]
use std::{println as info, println as warn, println as trace, println as debug};
use std::collections::hash_set::{IntoIter, Iter};
use std::sync::{Arc, Mutex};

pub enum ReportStrategy {
    NonEmptyContent,
    OnContentChange,
    OnWindowClose,
    Periodic(usize),
}

pub enum Tick {
    TimeDriven,
    TupleDriven,
    BatchDriven,
}

pub struct Report {
    strategies: Vec<ReportStrategy>,
    last_change: ContentGraph,
}

impl Report {
    pub fn new() -> Report {
        Report { strategies: Vec::new(), last_change: ContentGraph::new() }
    }
    pub fn add(&mut self, strategy: ReportStrategy) {
        self.strategies.push(strategy);
    }
    pub fn report(&mut self, window: &Window, content: &ContentGraph, ts: usize) -> bool {
        self.strategies.iter().all(|strategy| {
            match strategy {
                ReportStrategy::NonEmptyContent => content.len() > 0,
                ReportStrategy::OnContentChange => {
                    let comp = content.eq(&self.last_change);
                    self.last_change = content.clone();
                    comp
                }
                ReportStrategy::OnWindowClose => window.close < ts,
                ReportStrategy::Periodic(period) => ts % period == 0
            }
        })
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Window {
    open: usize,
    close: usize,
}
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct WindowTriple{
    pub s: String,
    pub p: String,
    pub o: String
}
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ContentGraph {
    elements: HashSet<WindowTriple>,
    last_timestamp_changed: usize,
}

impl ContentGraph {
    fn new() -> ContentGraph {
        ContentGraph { elements: HashSet::new(), last_timestamp_changed: 0 }
    }
    fn len(&self) -> usize {
        self.elements.len()
    }
    fn add(&mut self, triple: WindowTriple, ts: usize) {
        self.elements.insert(triple);
        self.last_timestamp_changed = ts;
    }
    fn get_last_timestamp_changed(&self) -> usize {
        self.last_timestamp_changed
    }

    pub fn iter(&self) -> Iter<'_, WindowTriple> {
        self.elements.iter()
    }
    pub fn into_iter(&mut self) -> IntoIter<WindowTriple> {
        let map = mem::take(&mut self.elements);
        map.into_iter()
    }
}


pub struct CSPARQLWindow {
    width: usize,
    slide: usize,
    t_0: usize,
    active_windows: HashMap<Window, ContentGraph>,
    report: Report,
    tick: Tick,
    app_time: usize,
    consumer: Option<Sender<ContentGraph>>
}

impl CSPARQLWindow {
    pub fn new(width:usize, slide: usize, report: Report, tick: Tick)-> CSPARQLWindow{
        CSPARQLWindow{slide, width, t_0: 0, app_time:0, report,consumer: None, active_windows: HashMap::new(),tick}
    }
    pub fn add_to_window(&mut self, triple: WindowTriple, ts: usize) {
        let event_time = ts;
        self.scope(&event_time);

        let test = self.active_windows.clone().into_iter().filter_map(|(window, mut content)| {
            debug!("Processing Window [{:?}, {:?}) for element ({:?},{:?})", window.open, window.close,triple, ts);
            if window.open <= event_time && event_time <= window.close {
                debug!("Adding element [{:?}] to Window [{:?},{:?})",triple, window.open, window.close);
                content.add(triple.clone(), ts);
                Some((window, content))
            } else {
                debug!("Scheduling for Eviction [{:?},{:?})", window.open, window.close);
                None
            }
        }).collect::<HashMap<Window, ContentGraph>>();


        let max = self.active_windows.iter()
            .filter(|(window, content)| self.report.report(window, content, ts))
            .max_by(|(w1, c1), (w2, c2)| w1.close.cmp(&w2.close));
        if let Some(max_window) = max {
            match self.tick {
                Tick::TimeDriven => {
                    if ts > self.app_time {
                        self.app_time = ts;
                        // notify consumers
                        debug!("Window triggers! {:?}", max_window);
                        if let Some(sender) = &self.consumer{
                            // thread::spawn(move ||{sender.send(max_window.1.clone())});
                            sender.send(max_window.1.clone());
                        }

                    }
                }
                _ => ()
            };
        }

        self.active_windows = test;
    }
    fn scope(&mut self, event_time: &usize) {
        // long c_sup = (long) Math.ceil(((double) Math.abs(t_e - t0) / (double) slide)) * slide;
        let temp = (*event_time as f64 - self.t_0 as f64).abs();
        let temp = ((*event_time as f64 - self.t_0 as f64).abs() / (self.slide as f64)).ceil();
        let c_sup = ((*event_time as f64 - self.t_0 as f64).abs() / (self.slide as f64)).ceil() * self.slide as f64;
        // long o_i = c_sup - width;
        let mut o_i = c_sup - self.width as f64;
        debug!("Calculating the Windows to Open. First one opens at [{:?}] and closes at [{:?}]", o_i, c_sup);
        // log.debug("Calculating the Windows to Open. First one opens at [" + o_i + "] and closes at [" + c_sup + "]");
        //
        loop {
            debug!("Computing Window [{:?},{:?}) if absent", o_i, (o_i +self.width as f64));
            let window = Window { open: o_i as usize, close: (o_i + self.width as f64) as usize };
            if let None = self.active_windows.get(&window) {
                self.active_windows.insert(window, ContentGraph::new());
            }
            o_i += self.slide as f64;
            if o_i > *event_time as f64 { break; }
        }
    }
    pub fn register(&mut self)-> Receiver<ContentGraph> {
        let (send, recv) = channel::<ContentGraph>();
        self.consumer.replace(send);
        recv
    }
    pub fn stop(&mut self){
        self.consumer.take();

    }
}
struct ConsumerInner{
    data: Mutex<Vec<ContentGraph>>
}
struct Consumer{
    inner: Arc<ConsumerInner>
}
impl Consumer{
    fn new() -> Consumer{
        Consumer{inner: Arc::new(ConsumerInner{data: Mutex::new(Vec::new())})}
    }
    fn start(&self,receiver: Receiver<ContentGraph>){
        let consumer_temp = self.inner.clone();
        thread::spawn(move||{
            loop{
                match receiver.recv(){
                    Ok(content)=> {
                        debug!("Found graph {:?}", content);
                        consumer_temp.data.lock().unwrap().push(content);
                    },
                    Err(_) => {
                        debug!("Shutting down!");
                        break;
                        }
                }
            }
        });
    }
    fn len(&self)->usize{
        self.inner.data.lock().unwrap().len()
    }
}
#[cfg(test)]
mod tests {
    use std::fmt::format;
    use std::sync::{Arc, Mutex};
    use std::thread::Thread;
    use crate::Encoder;
    use super::*;

    #[test]
    fn test_window() {
        let mut report = Report::new();
        report.add(ReportStrategy::OnWindowClose);
        let mut window = CSPARQLWindow { width: 10, slide: 2, app_time: 0, t_0: 0, active_windows: HashMap::new(), report, tick: Tick::TimeDriven, consumer: None };
        let receiver = window.register();
        let consumer = Consumer::new();
        consumer.start(receiver);
        let mut encoder = Encoder::new();

        for i in 0..10 {
            let triple = WindowTriple{s: format!("s{}", i), p: "p".to_string(), o: "o".to_string()};
            window.add_to_window(triple, i);
        }

        window.stop();
        thread::sleep(Duration::from_secs(1));
        assert_eq!(5, consumer.len());

    }


}