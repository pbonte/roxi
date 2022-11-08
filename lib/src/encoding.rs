use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;


static GLOBAL_ENCODER: Lazy<Mutex<InternalEncoder>> = Lazy::new(|| {Mutex::new(InternalEncoder::new())});

#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct InternalEncoder{
    encoded: HashMap<String, usize>,
    decoded: HashMap<usize,String>,
    counter: usize
}


impl InternalEncoder{
    fn new() -> InternalEncoder{
        InternalEncoder{encoded: HashMap::new(), decoded: HashMap::new(), counter:0}
    }
    fn add(&mut self, uri:String) -> usize{
        if let Some(encoded_uri) = self.encoded.get(&uri){
            return *encoded_uri;
        }else{
            self.encoded.insert(uri.clone(),self.counter);
            self.decoded.insert(self.counter,uri);
            self.counter+=1;
            self.counter -1
        }

    }
    fn get(&self, uri:&str) -> Option<usize>{
        if let Some(encoded_uri) = self.encoded.get(uri){
            return Some(*encoded_uri);
        }else{
            None
        }
    }

    fn decode(&self, encoded: &usize)->Option<&String>{
        self.decoded.get(encoded)
    }
}
#[derive(Debug)]
pub struct Encoder{}
impl Encoder{

    pub fn add(uri:String) -> usize{
        let mut encoder = GLOBAL_ENCODER.lock().unwrap();
        if let Some(encoded_uri) = encoder.encoded.get(&uri){
            return *encoded_uri;
        }else{
            let current_counter = encoder.counter;
            encoder.encoded.insert(uri.clone(),current_counter);
            encoder.decoded.insert(current_counter,uri);
            encoder.counter+=1;
            encoder.counter -1
        }

    }
    pub fn get(uri:&str) -> Option<usize>{
        let encoder = GLOBAL_ENCODER.lock().unwrap();
        if let Some(encoded_uri) = encoder.encoded.get(uri){
            return Some(*encoded_uri);
        }else{
            None
        }
    }

    pub fn decode(encoded: &usize)->Option<String>{
        let encoder = GLOBAL_ENCODER.lock().unwrap();
        let decoded = encoder.decoded.get(encoded);
        if let Some(decoded_value) = decoded{
            Some(decoded_value.clone())
        }else{
            None
        }
    }

}

#[test]
fn test_encoding(){
    let mut encoder = InternalEncoder::new();
    let _encoded1 = encoder.add("http://test/1".to_string());
    let encoded2 = encoder.add("http://test/2".to_string());
    let encoded3 = encoder.add("http://test/3".to_string());
    let decoded2 = encoder.decode(&encoded2);
    let decoded2_again = encoder.decode(&encoded2);
    assert_eq!("http://test/2", decoded2.unwrap());
    assert_eq!("http://test/2", decoded2_again.unwrap());
    assert_eq!(2,encoded3);
}