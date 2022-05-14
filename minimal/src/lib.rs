extern crate core;
pub mod ruleindex;
pub mod tripleindex;
use crate::ruleindex::RuleIndex;
use std::collections::HashMap;
use std::rc::Rc;
use crate::tripleindex::TripleIndex;

#[derive(Debug,  Clone, Eq, PartialEq)]
pub enum VarOrTerm{
    Var(Variable),
    Term(TermImpl)
}

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
    pub fn decode(&self, encoded: &usize)->Option<&String>{
        self.decoded.get(encoded)
    }
}
impl VarOrTerm{
    pub fn newTerm(iri:String, encoder: &mut Encoder) -> VarOrTerm{
        let  encoded = encoder.add(iri);
        VarOrTerm::Term(TermImpl{iri:encoded})
    }
    pub  fn newVar(name:String, encoder: &mut Encoder) -> VarOrTerm{
        let encoded = encoder.add(name);
        VarOrTerm::Var(Variable{name:encoded})
    }
    fn as_Term(&self) -> &TermImpl{
        if let VarOrTerm::Term(t) = self {t} else{ panic!("Not a term")}
    }
    fn as_Var(&self) -> &Variable{
        if let VarOrTerm::Var(v) = self {v} else{ panic!("Not a Var")}
    }
    fn is_var(&self) -> bool{
        match self {
            Self::Var(_) => true,
            Self::Term(_) => false,
    }}
    fn is_term(&self) -> bool {
        !self.is_var()
    }
    pub fn to_encoded(&self) -> usize {
        match self {
            Self::Var(var) => var.name,
            Self::Term(term) => term.iri,
        }
    }
}
#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct Variable{
    name: usize
}
#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct TermImpl {
    iri: usize
}
#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct Triple{
    pub s: VarOrTerm,
    pub p: VarOrTerm,
    pub o: VarOrTerm
}
#[derive(Debug,  Clone, Eq, PartialEq)]
pub struct Rule{
    pub body: Vec<Triple>,
    pub head: Triple
}
pub struct TripleStore<'a>{
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex<'a>,
    pub triple_index : TripleIndex,
    pub encoder: Encoder
}

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

        for left_c in (0..left.len()){
            for right_c in (0..right.len()){
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

}
impl <'a> TripleStore <'a>{
    pub fn new() -> TripleStore<'a>{
        TripleStore{rules: Vec::new(), rules_index: RuleIndex::new(), triple_index: TripleIndex::new(), encoder: Encoder::new() }
    }
    pub fn add(&mut self, triple: Triple){
        self.triple_index.add(triple);
    }
    pub fn len(&self) -> usize{
        self.triple_index.len()
}
    pub fn query(&self, query_triple:&Triple, triple_counter : Option<usize>) -> Binding{
        let mut bindings = Binding::new();
        let mut counter = if let Some(size) = triple_counter{size} else {self.triple_index.len()};
        for Triple{s,p,o} in self.triple_index.triples.iter().take(counter){
            match &query_triple.s{
                VarOrTerm::Var(s_var)=> bindings.add(&s_var.name,s.as_Term().iri),
                VarOrTerm::Term(s_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (s_term,s.as_Term()) {
                    if !iri.eq(iri2){break;}
                }
            }
            match &query_triple.p{
                VarOrTerm::Var(p_var)=> bindings.add(&p_var.name,p.as_Term().iri),
                VarOrTerm::Term(p_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (p_term,p.as_Term()) {
                    if !iri.eq(iri2){break;}
                }
            }
            match &query_triple.o{
                VarOrTerm::Var(o_var)=> bindings.add(&o_var.name,o.as_Term().iri),
                VarOrTerm::Term(o_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (o_term,o.as_Term()) {
                    if !iri.eq(iri2){break;}
                }
            }
        }
        bindings
    }
    fn query_with_join(&self, query_triples:&Vec::<Triple>,triple_counter : Option<usize>) -> Binding{
        let mut bindings = Binding::new();
        for query_triple in query_triples{
            //let current_bindings = self.query(query_triple,triple_counter);
            let current_bindings = self.triple_index.query(query_triple, triple_counter);
            bindings = bindings.join(&current_bindings);
        }
        bindings
    }
    pub fn materialize(&mut self) -> Vec<Triple>{
        let mut inferred = Vec::new();
        let mut counter = 0;
        while(counter < self.triple_index.triples.len()){
            let process_quad = self.triple_index.get(counter).unwrap();
            //let matching_rules = self.find_matching_rules(process_quad);
            let matching_rules = self.rules_index.find_match(process_quad);
            let mut new_triples = Vec::new();
            for rule in matching_rules{
                let temp_bindings = self.query_with_join(&rule.body,Some(counter + 1));
                let new_heads = self.subsititue_rule_head(&rule.head,&temp_bindings);
                for new_head in new_heads{
                    new_triples.push(new_head);

                }
            }
            for triple in new_triples{
                if !self.triple_index.contains(&triple) {
                    self.triple_index.add(triple);
                }
            }
            counter+=1;
        }

        inferred
    }
    fn subsititue_rule_head(&self, head: &Triple, binding: &Binding) ->Vec<Triple>{
        let mut new_heads = Vec::new();
        let mut s: &usize;
        let mut p: &usize;
        let mut o: &usize;
        for result_counter in 0..binding.len(){
            match &head.s{
                VarOrTerm::Var(s_var)=> s = binding.bindings.get(&s_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(s_term)=> s = &s_term.iri
            }
            match &head.p{
                VarOrTerm::Var(p_var)=> p = binding.bindings.get(&p_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(p_term)=> p = &p_term.iri
            }
            match &head.o{
                VarOrTerm::Var(o_var)=> o = binding.bindings.get(&o_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(o_term)=> o = &o_term.iri
            }
            new_heads.push(Triple{s:VarOrTerm::Term(TermImpl{iri:s.clone()}),p:VarOrTerm::Term(TermImpl{iri:p.clone()}),o:VarOrTerm::Term(TermImpl{iri:o.clone()})})
        }

        new_heads
    }
    fn find_matching_rules(&self, triple: &Triple) -> Vec<&Rule> {
        let mut matching_rules = Vec::new();
        for rule in self.rules.iter(){
            for body_item in rule.body.iter(){
                if let Triple{s,p,o} = triple{
                    match &body_item.s{
                        VarOrTerm::Term(s_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (s_term,s.as_Term()) {
                            if !iri.eq(iri2){break;}
                        },
                        _ => ()
                    }
                    match &body_item.p{
                        VarOrTerm::Term(p_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (p_term,p.as_Term()) {
                            if !iri.eq(iri2){break;}
                        },
                        _ => ()
                    }
                    match &body_item.o{
                        VarOrTerm::Term(o_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (o_term,o.as_Term()) {
                            if !iri.eq(iri2){break;}
                        },
                        _ => ()
                    }
                    if !matching_rules.contains(&rule){
                        matching_rules.push(rule);

                    }
                }
            }
        }
        matching_rules
    }

    fn parse_triple(data: &str, encoder: &mut Encoder) -> Triple{
        let items : Vec<&str> = data.split(" ").collect();
        let s = items.get(0).unwrap();
        let p = items.get(1).unwrap();
        let mut o_chars= items.get(2).unwrap().chars();
        o_chars.next_back();
        let o = o_chars.as_str();
        let mut convert_item = |item: &&str|{if item.starts_with("?"){VarOrTerm::newVar(item.to_string(),encoder)} else {VarOrTerm::newTerm(item.to_string(),encoder)}};
        let s = convert_item(s);
        let p = convert_item(p);
        let o = convert_item(&o);
        Triple{s,p,o}
    }
    fn rem_first_and_last(value: &str) -> &str {
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        chars.as_str()
    }
    fn parse( data: String, encoder: &mut Encoder) -> (Vec<Triple>, Vec<Rule>){
        let mut rules = Vec::new();
        let mut content = Vec::new();
        //line by line
        for line in data.split("\n"){
            if line.contains("=>"){
                //process rule
                let rule : Vec<&str> = line.split("=>").collect();
                let body = Self::rem_first_and_last(rule.get(0).unwrap());
                let head = Self::rem_first_and_last(rule.get(1).unwrap());
                let head_triple = Self::parse_triple(head,encoder);
                let mut body_triples = Vec::new();
                for body_triple in body.split("."){
                    body_triples.push(Self::parse_triple(body_triple, encoder));
                }
                rules.push(Rule{head:head_triple,body:body_triples})
            }else{
                //process triple
                let triple = Self::parse_triple(line,encoder);
                content.push(triple);
            }
        }
        (content, rules)
    }

}
#[cfg(test)]
mod tests {
    use crate::{Rule, Triple, TripleStore, TermImpl, VarOrTerm, RuleIndex, TripleIndex, Encoder};
    #[test]
    fn test_parse(){
        let mut encoder = Encoder::new();
        let data=":a a :A.\n\
        :b a :B.\n\
        {?a a :A}=>{?a a :C}\n\
        {?a a :B}=>{?a a :D}";
        let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);

        println!("Content {:?}", content);
        println!("Rules {:?}", rules);
        println!("encoded {:?}", encoder.decoded);

    }
    #[test]
    fn test_store() {
        let timer = ::std::time::Instant::now();
        let mut rules = Vec::new();
        let mut encoder = Encoder::new();
        let max_depth = 10;
        for i in 0..max_depth{
            let rule = Rule{head: Triple{s:VarOrTerm::newVar("s".to_string(), &mut encoder),p:VarOrTerm::newTerm("http://test".to_string(), &mut encoder),o:VarOrTerm::newTerm(format!("U{}", i+1), &mut encoder)},
                body: Vec::from([Triple{s:VarOrTerm::newVar("s".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm(format!("U{}",i),&mut encoder)}])};
            rules.push(rule);
        }

        let content =  Vec::from([Triple{s:VarOrTerm::newTerm("sTerm".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm("U0".to_string(),&mut encoder)}]);
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add(rule);
        }
        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let query = Triple{s:VarOrTerm::newVar("s".to_string(),&mut encoder),p:VarOrTerm::newTerm("http://test".to_string(),&mut encoder),o:VarOrTerm::newTerm(format!("U{}",max_depth),&mut encoder)};

        let mut store = TripleStore{rules:Vec::new(), rules_index , triple_index, encoder };

        store.materialize();
        let elapsed = timer.elapsed();

        let result = store.query_with_join(&Vec::from([query]), None);

        println!("Processed in: {:.2?}", elapsed);
        println!("Result: {:?}", result);

    }
    #[test]
    fn test_encoding(){
        let mut encoder = Encoder::new();
        let encoded1 = encoder.add("http://test/1".to_string());
        let encoded2 = encoder.add("http://test/2".to_string());
        let encoded3 = encoder.add("http://test/3".to_string());
        let dedocded1 = encoder.decode(&encoded1);
        let dedocded2 = encoder.decode(&encoded2);
        let dedocded2_2 = encoder.decode(&encoded2);
        assert_eq!("http://test/2",dedocded2.unwrap());
        assert_eq!("http://test/2",dedocded2_2.unwrap());
        assert_eq!(2,encoded3);
    }
}
