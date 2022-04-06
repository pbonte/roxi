use std::fmt;
use std::fmt::Formatter;
use std::convert::{TryFrom, TryInto};

enum StreamStatus{
    Streaming,
    Paused,
    Ended
}
fn inspect_stream( status:StreamStatus){
    match status{
        StreamStatus::Streaming => println!("Streaming"),
        StreamStatus::Paused => println!("Pauzed"),
        StreamStatus::Ended => println!("Ended"),
    }
}
struct Test(i32);

impl fmt::Display for Test{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"TESTER{}",self.0)
    }
}
fn borrow_slice(slice: &[i32]){
    println!("{}",slice[0]);
}
#[derive(Debug)]
struct Point{
    x: f32,
    y: f32
}
impl Point{
    fn origin() -> Point{
        Point{x:0.0,y:0.0}
    }
}
#[derive(Debug)]
struct Number{
    value: i32
}
// impl From<i32> for Number{
//     fn from(item: i32) -> Self {
//         Number { value:item }
//     }
// }
impl TryFrom<i32> for Number{
    type Error = ();

    fn try_from(valuee: i32) -> Result<Self, Self::Error> {
        Ok(Number{value:valuee})
    }
}
fn borrow_value(b: &mut Box<i32>){
    **b+=10;
    println!("boorow and estroed {}", b);
}
fn main() {
    let mut p = Point{x:0.0,y:0.0};
    let p_borrow = &p;
    let p_borrow2 = &p;
    println!("{:?}", p_borrow);
    let mut p_mut_borrow = &mut p;
    p_mut_borrow.x = 10.0;
    println!("{:?}", p);

    let mut test = Box::new(5i32);
    let mut test2 = test;
    //let tt:&i32 = &test;
    *test2 = 4;
    borrow_value(&mut test2);
    println!("{}",test2);
    let mut counter = 0;
    let loop_test = loop{
        counter+=1;
        if counter == 10{
            break counter;
        }
    };
    println!("{}",loop_test);
    let r:Result<Number,()> = 3i32.try_into();
    println!("{:?}",r);
    let t:i32 = 90;
    // let m:Number = t.into();
    // println!("{:?}",m);

    let flloat = 3.45;
    let iint = flloat as u8;
    println!("{}",iint);
    let p = Point{x:0.0,y:0.9};
    let p2 =Point{x:0.9,..p};
    let p3 = Point::origin();
    println!("{:?}",p3);
    let status = StreamStatus::Streaming;
    inspect_stream(status);
}
