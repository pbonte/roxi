//! Observer is a behavioral design pattern that allows one objects to notify other objects about changes in their state.

trait IObserver {
    fn update(&self, new: Vec<i32>) -> Vec<i32>;
}

trait ISubject<'a, T: IObserver> {
    fn attach(&mut self, observer: &'a T);
    fn detach(&mut self, observer: &'a T);
    fn notify_observers(&mut self);
    fn add_data(&mut self, data: i32);
}

struct Subject<'a, T: IObserver> {
    observers: Vec<&'a T>,
    data: Vec<i32>
}
impl<'a, T: IObserver + PartialEq> Subject<'a, T> {
    fn new() -> Subject<'a, T> {
        Subject {
            observers: Vec::new(),
            data: Vec::new()
        }
    }
}

impl<'a, T: IObserver + PartialEq> ISubject<'a, T> for Subject<'a, T> {
    fn attach(&mut self, observer: &'a T) {
        self.observers.push(observer);
    }
    fn detach(&mut self, observer: &'a T) {
        if let Some(idx) = self.observers.iter().position(|x| *x == observer) {
            self.observers.remove(idx);
        }
    }
    fn notify_observers(&mut self) {
        for item in self.observers.iter() {
            let updates_received = item.update(self.data.clone());
            updates_received.into_iter().for_each(|u| self.data.push(u));
        }
    }

    fn add_data(&mut self, data: i32) {
        self.data.push(data);
    }
}

#[derive(PartialEq)]
struct ConcreteObserver {
    id: i32,
    data: i32
}
impl IObserver for ConcreteObserver {
    fn update(&self, new: Vec<i32>) -> Vec<i32>{
        println!("Observer id:{} received event with data {:?}!", self.id, new);
        vec![self.data]
    }

}
impl ConcreteObserver{
    fn update_data(&mut self, new_data:i32){
        self.data = new_data;
    }
}
#[test]
fn test_observer() {
    let mut subject = Subject::new();
    let observer_a = ConcreteObserver { id: 1, data:0 };
    let observer_b = ConcreteObserver { id: 2 , data:0};

    subject.attach(&observer_a);
    subject.attach(&observer_b);
    subject.add_data(1337);
    subject.notify_observers();
    subject.add_data(1339);
    subject.detach(&observer_b);
    subject.notify_observers();

}