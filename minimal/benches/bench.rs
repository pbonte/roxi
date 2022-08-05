#[macro_use]
extern crate bencher;

use std::cell::RefCell;
use std::rc::Rc;
use bencher::Bencher;

use minimal::imars::{ImarsWindow,SimpleWindowConsumer};

fn a(bench: &mut Bencher) {
    bench.iter(|| {
        (0..1000).fold(0, |x, y| x + y)
    })
}

fn b(bench: &mut Bencher) {
    const N: usize = 1024;
    bench.iter(|| {
        vec![0u8; N]
    });

    bench.bytes = N as u64;
}

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
fn add_10000(bench: &mut Bencher) {
   start_add_test(bench,1000,10,10000);
}
fn add_100000(bench: &mut Bencher) {
    start_add_test(bench,1000,10,100000);

}
fn add_1000000(bench: &mut Bencher) {
    start_add_test(bench,1000,10,1000000);

}

benchmark_group!(benches, add_10000);
benchmark_main!(benches);