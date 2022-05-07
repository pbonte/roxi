use std::collections::HashMap;
use oxigraph::model::Term;

#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct Binding{
    bindings: HashMap<String,Vec<Term>>
}




impl Binding {
    pub fn new() -> Binding {
        Binding{bindings:HashMap::new()}
    }
    pub fn add(&mut self, var_name: &str, term: Term) {
        if !self.bindings.contains_key(var_name){
            self.bindings.insert(var_name.to_string(), Vec::new());
        }
        let mut binding_values= self.bindings.get_mut(var_name).unwrap();
        binding_values.push(term);
    }
    pub  fn contains(&self, key: &str) -> bool {
       self.bindings.contains_key(key)
    }
    pub fn get(&self, var_name: &str) -> Vec<Term> {
        match self.bindings.get(var_name){
            Some(values) => values.clone(),
            _ => Vec::new()
        }
    }
    pub fn len(&self) -> usize{
        if let Some(values) = self.bindings.values().into_iter().next(){
            return values.len();
        }
        0
    }
    pub fn join(&self, join_binding: &Binding) -> Binding {
        let mut left = self;
        let mut right = join_binding;
        if left.len() == 0 {return right.clone();}
        if right.len() == 0 {return left.clone();}
        let mut result = Binding::new();
        if left.len()<right.len(){
            right = self;
            left = join_binding;
        }
        //find join keys
        let join_keys:Vec<&String>= left.bindings.keys().into_iter().filter(|k|right.bindings.contains_key(*k)).collect();

        for left_c in (0..left.len()){
            for right_c in (0..right.len()){
                // iterate over all join keys
                let mut match_keys=true;
                for join_key in &join_keys{
                    let left_term = left.bindings.get(*join_key).unwrap().get(left_c).unwrap();
                    let right_term = right.bindings.get(*join_key).unwrap().get(right_c).unwrap();
                    if !left_term.eq(right_term){
                        match_keys = false;
                        break;
                    }
                }
                if match_keys{
                    left.bindings.keys().into_iter()
                        .for_each(|k|result.add(k,left.bindings.get(k).unwrap().get(left_c).unwrap().clone()));
                    //add right data (without the current key
                    right.bindings.keys().into_iter()
                        .filter(|k|!left.bindings.contains_key(*k))
                        .for_each(|k|result.add(k,right.bindings.get(k).unwrap().get(right_c).unwrap().clone()));
                }
            }
        }
        result
    }
    pub fn project(&mut self, projection_keys: Vec::<&str>) -> Binding {
        let mut projection = Binding::new();
        for key in projection_keys{
            if self.bindings.contains_key(key) {
                projection.bindings.insert(key.to_string(), self.bindings.get(key).unwrap().clone());
            }

        }
        projection
    }
    pub fn rename(&self, var_subs: Vec<(String, String)>) -> Binding {
        let mut renamed = Binding::new();
        for (orig_name, new_name) in var_subs{
            renamed.bindings.insert(new_name,self.bindings.get(orig_name.as_str()).unwrap().clone());
        }
        renamed
    }
    pub fn combine(&mut self, to_combine: Binding) {
        for (k,v) in to_combine.bindings{
            if !self.bindings.contains_key(&k){
                self.bindings.insert(k.to_string(),Vec::new());
            }
            let mut add_vec = self.bindings.get_mut(k.as_str()).unwrap();
            for value in v{
                add_vec.push(value);
            }
        }
    }
}
#[cfg(test)]
mod tests{
    use oxigraph::model::{NamedNode, Term};
    use crate::reasoningstore::binding::Binding;
    use crate::reasoningstore::triple::ReasonerTriple;

    #[test]
    fn test_bindings(){

        let rule_atom = ReasonerTriple::new("?s".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SubClass".to_string());

        let mut binding: Binding = Binding::new();
        binding.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        let result:Vec<Term> = binding.get("s");
        assert_eq!(result.len(), 1);
        let result:Vec<Term> = binding.get("new_present_var");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_simple_join(){
        let mut left: Binding = Binding::new();
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        let mut right: Binding = Binding::new();
        right.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));

        let join_binding:Binding = left.join(&right);

        let mut result: Binding = Binding::new();
        result.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        result.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));
        result.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));
        assert_eq!(result, join_binding);

    }
    #[test]
    fn test_single_join_multiple_values(){
        let mut left: Binding = Binding::new();
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass2".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass2".to_string()).unwrap()));

        let mut right: Binding = Binding::new();
        right.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        right.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass2".to_string()).unwrap()));

        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));
        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass2".to_string()).unwrap()));

        let join_binding:Binding = left.join(&right);

        let mut result: Binding = Binding::new();
        result.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        result.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass2".to_string()).unwrap()));

        result.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));
        result.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass2".to_string()).unwrap()));

        result.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));
        result.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass2".to_string()).unwrap()));

        assert_eq!(result, join_binding);

    }
    #[test]
    fn test_single_join_multiple_values_single_match(){
        let mut left: Binding = Binding::new();
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass2".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass2".to_string()).unwrap()));

        let mut right: Binding = Binding::new();
        right.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        right.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass3".to_string()).unwrap()));

        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));
        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass2".to_string()).unwrap()));

        let join_binding:Binding = left.join(&right);

        let mut result: Binding = Binding::new();
        result.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        result.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        result.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));

        assert_eq!(result, join_binding);

    }
    #[test]
    fn test_single_uneven_join(){
        let mut left: Binding = Binding::new();
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass2".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass2".to_string()).unwrap()));

        let mut right: Binding = Binding::new();
        right.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));

        let join_binding:Binding = left.join(&right);

        let mut result: Binding = Binding::new();
        result.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        result.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));
        result.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeOtherClass".to_string()).unwrap()));

        assert_eq!(result, join_binding);

    }
    #[test]
    fn test_multiple_join(){
        let mut left: Binding = Binding::new();
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        let mut right: Binding = Binding::new();
        right.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        right.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        let join_binding:Binding = left.join(&right);

        let mut result: Binding = Binding::new();
        result.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        result.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        assert_eq!(result, join_binding);

    }
    #[test]
    fn test_no_match_join(){
        let mut left: Binding = Binding::new();
        left.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        left.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        let mut right: Binding = Binding::new();
        right.add("t", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        let join_binding:Binding = left.join(&right);

        let mut result: Binding = Binding::new();
        result.add("s", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));
        result.add("p", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));
        result.add("t", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        result.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        assert_eq!(result, join_binding);

    }
    #[test]
    fn test_empty_binding_join(){
        let mut left: Binding = Binding::new();


        let mut right: Binding = Binding::new();
        right.add("t", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        right.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        let join_binding:Binding = left.join(&right);

        let mut result: Binding = Binding::new();
        result.add("t", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        result.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        assert_eq!(result, join_binding);

    }
    #[test]
    fn test_project_bindings(){
        let mut binding: Binding = Binding::new();
        binding.add("t", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));

        binding.add("q", Term::from(NamedNode::new("http://www.test.be/test#SomeClass".to_string()).unwrap()));

        let projection = binding.project(Vec::from(["t"]));

        let mut result: Binding = Binding::new();
        result.add("t", Term::from(NamedNode::new("http://www.test.be/test#SubClass".to_string()).unwrap()));


        assert_eq!(result, projection);
    }
}