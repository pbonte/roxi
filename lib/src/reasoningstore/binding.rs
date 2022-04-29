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
        let mut result = Binding::new();
        if left.len()<right.len(){
            right = self;
            left = join_binding;
        }
        //find join keys
        let join_keys = left.bindings.keys().into_iter().filter(|k|right.bindings.contains_key(*k));
        for join_key in join_keys{
            for left_c in (0..left.bindings.get(join_key).unwrap().len()){
                for right_c in (0..right.bindings.get(join_key).unwrap().len()){
                    if left.bindings.get(join_key).unwrap().get(left_c).unwrap().eq(
                        right.bindings.get(join_key).unwrap().get(right_c).unwrap()){

                    }
                }
            }
        }
        result
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
}