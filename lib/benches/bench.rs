#[macro_use]
extern crate bencher;

use std::cell::RefCell;
use std::rc::Rc;
use bencher::Bencher;

use minimal::imars_window::{ImarsWindow, SimpleWindowConsumer};
use minimal::time_window::TimeWindow;
use minimal::parser::Parser;
use minimal::pipeline::WindowReasoner;
use minimal::TripleStore;

fn create_window(width:i32, slide:i32) -> ImarsWindow<i32> {
    let mut window :ImarsWindow<i32> = ImarsWindow::new(width,slide);
    let consumer = Rc::new(RefCell::new(SimpleWindowConsumer::new()));
    window.register_consumer(consumer.clone());
    window
}
fn add(window: &mut ImarsWindow<i32>, size:i32){
    for i in 1..size{
        window.add(i,i);
    }
}
fn start_add_test(bench: &mut Bencher, width: i32, slide:i32, size:i32){
    let mut window = create_window(width,slide);
    bench.iter(|| {
        add(&mut window,size)
    });
}
fn add_100(bench: &mut Bencher) {
   start_add_test(bench,100,10,100000);
}
fn add_1000(bench: &mut Bencher) {
    start_add_test(bench,1000,10,100000);

}
fn add_10000(bench: &mut Bencher) {
    start_add_test(bench,10000,10,100000);
}


fn update(window: &mut ImarsWindow<i32>, width:i32, size:i32){
    for i in 1..size{
        if i / width > 0 && (i % width) == width - width/10  {
            window.add(i-width/2,i);
        }else{
            window.add(i,i);
        }
    }
}
fn start_update_test(bench: &mut Bencher, width: i32, slide:i32, size:i32){
    let mut window = create_window(width,slide);
    bench.iter(|| {
        update(&mut window,width,size);
    });
}
fn update_100(bench: &mut Bencher) {
    start_update_test(bench,100,10,100000);
}
fn update_1000(bench: &mut Bencher) {
    start_update_test(bench,1000,10,100000);

}
fn update_10000(bench: &mut Bencher) {
    start_update_test(bench,10000,10,100000);
}

// fn pipeline(bench: &mut Bencher){
//     bench.iter(|| {
//         let mut window = TimeWindow::new(10, 5);
//
//         let mut data = "{?a in ?b.?b in ?c}=>{?a in ?c}\n".to_owned();
//         for i in 0..500{
//             data += format!(":{} in :{}.\n",i+1,i).as_str();
//         }
//
//
//
//         let mut reasoner = WindowReasoner::new();
//
//         let (mut content, mut rules) = Parser::parse(data.to_string(), &mut reasoner.store.encoder);
//         reasoner.store.add_rules(rules);
//         let mut consumer = Rc::new(RefCell::new(reasoner));
//         window.register_consumer(consumer.clone());
//
//
//         content.into_iter().enumerate().for_each(|(i, t)| window.add(t, i as i32));
//     });
//
// }
fn test_transitive_rule(bench: &mut Bencher){
    bench.iter(|| {
        let mut data = "{?a in ?b.?b in ?c}=>{?a in ?c}\n".to_owned();
        for i in 0..100 {
            data += format!(":{} in :{}.\n", i + 1, i).as_str();
        }
        let mut store = TripleStore::from(data.as_str());
        store.materialize();
    });
}
benchmark_group!(benches, test_transitive_rule);
benchmark_main!(benches);