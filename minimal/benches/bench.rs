#[macro_use]
extern crate bencher;

use std::cell::RefCell;
use std::rc::Rc;
use bencher::Bencher;

use minimal::imars::{ImarsWindow,SimpleWindowConsumer};

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
benchmark_group!(benches, update_100,update_1000,update_10000,add_100,add_1000,add_10000);
benchmark_main!(benches);