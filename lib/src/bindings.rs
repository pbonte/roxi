use std::collections::HashMap;
use std::path::Iter;

#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct Binding {
    bindings: HashMap<usize, Vec<usize>>,
}
impl Binding  {
    pub fn new() -> Binding {
        Binding { bindings: HashMap::new() }
    }
    pub fn add(& mut self, var_name: &usize, term: usize) {
        if !self.bindings.contains_key(var_name){
            self.bindings.insert(*var_name, Vec::new());
        }
        let mut binding_values= self.bindings.get_mut(var_name).unwrap();
        binding_values.push(term);
    }
    pub fn len(&self) -> usize{
        if let Some(values) = self.bindings.values().into_iter().next(){
            return values.len();
        }
        0
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, usize, Vec<usize>> {
        self.bindings.iter()
    }
    pub fn get(&self,key:&usize)->Option<&Vec<usize>>{
        self.bindings.get(key)
    }
    pub fn join(& self, join_binding: & Binding) -> Binding {
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
        let join_keys:Vec<&usize>= left.bindings.keys().into_iter().filter(|k|right.bindings.contains_key(*k)).collect();

        for left_c in 0..left.len(){
            for right_c in 0..right.len(){
                // iterate over all join keys
                let mut match_keys=true;
                for join_key in &join_keys{
                    let left_term = left.bindings.get(*join_key).unwrap().get(left_c).unwrap();
                    let right_term = right.bindings.get(*join_key).unwrap().get(right_c).unwrap();
                    if left_term != right_term{
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
    pub fn combine(&mut self, to_combine: Binding) {
        let binding_size = self.bindings.values().map(|v|v.len()).max().unwrap_or(1);
        for (k,v) in to_combine.bindings{
            if !self.bindings.contains_key(&k){
                self.bindings.insert(k,Vec::new());
            }
            let mut add_vec = self.bindings.get_mut(&k).unwrap();
            for value in v{
                for _ in 0..binding_size {
                    add_vec.push(value);
                }
            }
        }
    }
    pub fn rename(&self, var_subs: Vec<(usize, usize)>) -> Binding {
        let mut renamed = Binding::new();
        for (orig_name, new_name) in var_subs{
            if let Some(bound_value) = self.bindings.get(&orig_name){
                renamed.bindings.insert(new_name,bound_value.clone());
            }

        }
        renamed
    }
    pub fn remove_vars(&mut self, var_names: &[usize]) {
        for var_name in var_names{
            self.bindings.remove(var_name);
        }
    }
    pub fn retain_vars(&mut self, var_names: &[usize]) {
        self.bindings.retain(|k,_|var_names.contains(k));
    }
    pub fn vars (&self) -> Vec<&usize>{
       self.bindings.keys().collect()
    }

}