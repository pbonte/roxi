use std::collections::HashMap;

#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct Encoder{
    encoded: HashMap<String, usize>,
    decoded: HashMap<usize,String>,
    counter: usize
}
impl Encoder{
    pub fn new() -> Encoder{
        Encoder{encoded: HashMap::new(), decoded: HashMap::new(), counter:0}
    }
    pub fn add(&mut self, uri:String) -> usize{
        if let Some(encoded_uri) = self.encoded.get(&uri){
            return *encoded_uri;
        }else{
            self.encoded.insert(uri.clone(),self.counter);
            self.decoded.insert(self.counter,uri);
            self.counter+=1;
            self.counter -1
        }

    }
    pub fn get(&self, uri:&str) -> Option<usize>{
        if let Some(encoded_uri) = self.encoded.get(uri){
            return Some(*encoded_uri);
        }else{
            None
        }
    }

    pub fn decode(&self, encoded: &usize)->Option<&String>{
        self.decoded.get(encoded)
    }
}
#[test]
fn test_encoding(){
    let mut encoder = Encoder::new();
    let _encoded1 = encoder.add("http://test/1".to_string());
    let encoded2 = encoder.add("http://test/2".to_string());
    let encoded3 = encoder.add("http://test/3".to_string());
    let decoded2 = encoder.decode(&encoded2);
    let decoded2_again = encoder.decode(&encoded2);
    assert_eq!("http://test/2", decoded2.unwrap());
    assert_eq!("http://test/2", decoded2_again.unwrap());
    assert_eq!(2,encoded3);
}