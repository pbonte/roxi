extern crate core;
pub mod ruleindex;
pub mod tripleindex;
pub mod imars;
pub mod imars_reasoner;
pub mod bindings;
use crate::ruleindex::RuleIndex;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::tripleindex::TripleIndex;
use std::fmt::Write;
use crate::bindings::Binding;

#[derive(Debug,  Clone, Eq, PartialEq, Hash)]
pub enum VarOrTerm{
    Var(Variable),
    Term(TermImpl)
}
#[derive(Debug)]
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
#[derive(Debug,  Clone, Eq, PartialEq, Hash)]
pub struct Variable{
    name: usize
}
#[derive(Debug,  Clone, Eq, PartialEq, Hash)]
pub struct TermImpl {
    iri: usize
}
#[derive(Debug,  Clone, Eq, PartialEq, Hash)]
pub struct Triple{
    pub s: VarOrTerm,
    pub p: VarOrTerm,
    pub o: VarOrTerm
}
#[derive(Debug,  Clone, Eq, PartialEq, Hash)]
pub struct Rule{
    pub body: Vec<Triple>,
    pub head: Triple
}
pub struct TripleStore{
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index : TripleIndex,
    pub encoder: Encoder
}

impl TripleStore {

}


impl TripleStore {
    pub fn new() -> TripleStore{
        TripleStore{rules: Vec::new(), rules_index: RuleIndex::new(), triple_index: TripleIndex::new(), encoder: Encoder::new() }
    }
    pub fn add(&mut self, triple: Triple){
        self.triple_index.add(triple);
    }
    pub fn add_ref(&mut self, triple: Rc<Triple>){
        self.triple_index.add_ref(triple);
    }
    pub(crate) fn add_rules(&mut self, rules: Vec<Rule>) {
        rules.into_iter().for_each(|rule|self.rules_index.add(rule));
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
                    inferred.push(triple.clone());
                    self.triple_index.add(triple);
                }
            }
            counter+=1;
        }

        inferred
    }
    //Backward chaining
    fn eval_backward(&self, rule_head: &Triple)->Binding{
        let sub_rules : Vec<(Rc<Rule>, Vec<(usize, usize)>)> = self.find_subrules(rule_head);
        let mut all_bindings = Binding::new();
        for (sub_rule,var_subs) in sub_rules.into_iter(){
            let mut rule_bindings = Binding::new();
            for rule_atom in &sub_rule.body{
                let result_bindings = self.triple_index.query(rule_atom,None);
                rule_bindings = rule_bindings.join(&result_bindings);
                //recursive call
                let recursive_bindings = self.eval_backward(rule_atom);
                rule_bindings.combine(recursive_bindings);
            }
            //rename variables
            let renamed = rule_bindings.rename(var_subs);
            all_bindings.combine(renamed);
        }
        all_bindings
    }
    pub(crate) fn compute_sprite(&mut self, query: &Triple) {
        let  (backward_rules, hierarcies) = self.eval_backward_csprite( query);

        // new rules
        let mut new_rules: Vec<Rc<Rule>> = backward_rules.into_iter().filter(|r|r.body.len()>1).collect();
        for hierarchy in hierarcies{
            let rewritten_hierarchy = rewrite_hierarchy(&hierarchy);
            rewritten_hierarchy.into_iter().for_each(|r|new_rules.push(Rc::new(r)));
        }

        // new rule index
        let mut parsed_rules_index = RuleIndex::new();
        for rule in new_rules.iter(){
            parsed_rules_index.add_ref(rule);
        }
        self.rules_index = parsed_rules_index;
    }
    fn eval_backward_csprite(&self, rule_head: &Triple)->(HashSet<Rc<Rule>>, Vec<Vec<Rc<Rule>>>){
        //TODO check cycles
        // let mut matched_rules = HashSet::new();
        // let mut hierarchies = Vec::new();
        // self.eval_backward_csprite_helper(rule_head,&mut matched_rules,false, &mut hierarchies);
        // (matched_rules, hierarchies)
        self.eval_backward_csprite_helper_with_stack(rule_head)
    }
    fn eval_backward_csprite_helper(&self, rule_head: &Triple, matched_rules: &mut HashSet<Rc<Rule>>, hierarchy:bool, hierarchies: &mut Vec<Vec<Rc<Rule>>>){
        //TODO check cycles
        let sub_rules : Vec<(Rc<Rule>, Vec<(usize, usize)>)> = self.find_subrules(rule_head);
        let mut current_hierarchy= false;
        for (sub_rule,var_subs) in sub_rules.into_iter(){
            if matched_rules.insert(sub_rule.clone()) {
                if sub_rule.body.len() == 1{
                    //hierarchy candidate
                    if hierarchy{
                        if let Some(current_hierarchy) = hierarchies.last_mut(){
                            current_hierarchy.push(sub_rule.clone());
                        }
                    }
                    else{
                        hierarchies.push(Vec::from([sub_rule.clone()]));
                    }
                    current_hierarchy = true;
                }
                for rule_atom in &sub_rule.body {
                    //recursive call
                   self.eval_backward_csprite_helper(rule_atom,matched_rules,current_hierarchy, hierarchies);
                }
            }

        }
    }
    fn eval_backward_csprite_helper_with_stack(&self, rule_head: &Triple)->(HashSet<Rc<Rule>>, Vec<Vec<Rc<Rule>>>){
        //TODO check cycles
        let mut stack = Vec::from([rule_head.clone()]); //TODO add initial size & pointers instead of triples
        let mut matched_rules = HashSet::new();
        let mut hierarchies: Vec<Vec<Rc<Rule>>> = Vec::new();
        let mut hierarchy = false;
        while !stack.is_empty() {
            let current_head = stack.pop().unwrap();
            let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> = self.find_subrules(&current_head);
            let mut current_hierarchy = false;
            for (sub_rule, var_subs) in sub_rules.into_iter() {
                if matched_rules.insert(sub_rule.clone()) {
                    if sub_rule.body.len() == 1 {
                        //hierarchy candidate
                        if hierarchy {
                            if let Some(current_hierarchy) = hierarchies.last_mut() {
                                current_hierarchy.push(sub_rule.clone());
                            }
                        } else {
                            hierarchies.push(Vec::from([sub_rule.clone()]));
                        }
                        current_hierarchy = true;
                    }
                    for rule_atom in &sub_rule.body {
                        //recursive call
                        //self.eval_backward_csprite_helper(rule_atom,matched_rules,current_hierarchy, hierarchies);
                        stack.push(rule_atom.clone());
                        hierarchy = current_hierarchy;
                    }
                }
            }
        }
        (matched_rules, hierarchies)
    }

    pub(crate) fn find_subrules(&self, rule_head: &Triple) -> Vec<(Rc<Rule>,Vec<(usize,usize)>)> {
        let mut rule_matches = Vec::new();
        for rule in self.rules_index.rules.iter(){
            let head:&Triple = &rule.head;
            let mut var_names_subs  :Vec::<(usize,usize)>= Vec::new();
            if self.eval_triple_element(&head.s, &rule_head.s, &mut var_names_subs) &&
                self.eval_triple_element(&head.p,&rule_head.p,&mut var_names_subs) &&
                self.eval_triple_element(&head.o,&rule_head.o,&mut var_names_subs) {
                rule_matches.push((rule.clone(), var_names_subs));
            }
        }
        rule_matches
    }
    fn eval_triple_element(&self, left: &VarOrTerm, right:&VarOrTerm,   var_names_sub: &mut Vec<(usize, usize)>) -> bool{
        if let (VarOrTerm::Var(left_name) ,VarOrTerm::Var(right_name))= (left,right) {
            var_names_sub.push((left_name.name, right_name.name));
            true
        }else{
            left.eq(right)
        }
    }
    ////
    fn subsititue_rule_head(&self, head: &Triple, binding: &Binding) ->Vec<Triple>{
        let mut new_heads = Vec::new();
        let mut s: &usize;
        let mut p: &usize;
        let mut o: &usize;
        for result_counter in 0..binding.len(){
            match &head.s{
                VarOrTerm::Var(s_var)=> s = binding.get(&s_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(s_term)=> s = &s_term.iri
            }
            match &head.p{
                VarOrTerm::Var(p_var)=> p = binding.get(&p_var.name).unwrap().get(result_counter).unwrap(),
                VarOrTerm::Term(p_term)=> p = &p_term.iri
            }
            match &head.o{
                VarOrTerm::Var(o_var)=> o = binding.get(&o_var.name).unwrap().get(result_counter).unwrap(),
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

        let o = if items.get(2).unwrap().ends_with("."){
            let mut o_chars= items.get(2).unwrap().chars();
            o_chars.next_back();
            o_chars.as_str()
        }else{
            items.get(2).unwrap()
        };
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
    pub fn parse(data: String, encoder: &mut Encoder) -> (Vec<Triple>, Vec<Rule>){
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
                    if body_triple.len() >0 {
                        body_triples.push(Self::parse_triple(body_triple, encoder));
                    }
                }
                rules.push(Rule{head:head_triple,body:body_triples})
            }else{
                //process triple
                if line.len() > 0 {
                    let triple = Self::parse_triple(line, encoder);
                    content.push(triple);
                }
            }
        }
        (content, rules)
    }

    fn decode_triples(triples: &Vec<Triple>, encoder: &Encoder) -> String {
        let mut res = String::new();
        for triple in triples {
            let decoded_s = encoder.decode(&triple.s.to_encoded()).unwrap();
            let decoded_p = encoder.decode(&triple.p.to_encoded()).unwrap();
            let decoded_o = encoder.decode(&triple.o.to_encoded()).unwrap();

            write!(&mut res, "{} {} {}.\n", decoded_s, decoded_p, decoded_o).unwrap();
        }
        res
    }
    pub fn content_to_string(&mut self) -> String{
        let content = &self.triple_index.triples;
        let encoder = &self.encoder;
        TripleStore::decode_triples(content,encoder)
    }

}
fn rewrite_hierarchy(rules: &Vec<Rc<Rule>>) -> Vec<Rule>{
    let mut new_rules = Vec::new();
    if rules.len() >0 {
        let new_head = &rules.get(0).unwrap().head;
        for rule in rules.iter(){
            new_rules.push(Rule{body: rule.body.clone(), head: new_head.clone()})
        }
    }

    new_rules
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::{Rule, Triple, TripleStore, TermImpl, VarOrTerm, RuleIndex, TripleIndex, Encoder, rewrite_hierarchy};
    #[test]
    fn test_parse(){
        let mut encoder = Encoder::new();
        let data=":a a :C0.\n\
            {?a a :C0}=>{?a a :C1}\n\
            {?a a :C1}=>{?a a :C2}\n\
            {?a a :C2}=>{?a a :C3}";
        let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);

        println!("Content {:?}", content);
        println!("Rules {:?}", rules);
        println!("encoded {:?}", encoder.decoded);

        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add_ref(rule);
        }
        let mut store = TripleStore{rules:Vec::new(), rules_index , triple_index, encoder };

        let mat = store.materialize();
        println!("Length: {:?}", store.len());
        println!("Length Mat: {:?}", mat.len());
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
            rules_index.add_ref(rule);
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
    #[test]
    fn test_eval_backward_rule(){
        let mut encoder = Encoder::new();
        let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClass.\n\
            {?s a test:SubClass.}=>{?s a test:SubClass2.}\n
            {?s a test:SubClass2.?s test:hasRef ?b.?b test:hasRef ?c.?c a test:SubClass2.}=>{?s a test:SuperType.}";
        let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);

        println!("Content {:?}", content);
        println!("Rules {:?}", rules);


        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add_ref(rule);
        }
        let backward_head = Triple{s:VarOrTerm::newVar("?newVar".to_string(),&mut encoder),p:VarOrTerm::newTerm("a".to_string(),&mut encoder),o:VarOrTerm::newTerm("test:SuperType".to_string(),&mut encoder)};
        let var_encoded= encoder.add("?newVar".to_string());
        let result_encoded = encoder.add("<http://example2.com/a>".to_string());
        println!("encoded {:?}", encoder.decoded);
        let mut store = TripleStore{rules:rules, rules_index , triple_index, encoder };
        //        let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());

        let  bindings = store.eval_backward( &backward_head);
        let result_bindings = HashMap::from([
            (var_encoded, Vec::from([result_encoded]))
        ]);
        println!("{:?}",bindings);
        assert_eq!(result_bindings.get(&12), bindings.get(&12));
    }
    #[test]
    fn test_sprite_compute(){
        let mut encoder = Encoder::new();
        let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClass.\n\
            {?s a test:SubClass.}=>{?s a test:SubClass2.}\n\
            {?s a test:SubClass2.}=>{?s a test:SubClass.}\n\
            {?s a test:SubClass0.}=>{?s a test:SubClass2.}\n\
            {?s a test:SubClass01.}=>{?s a test:SubClass0.}\n\
            {?s a test:SubClassH1.}=>{?s a test:SubClassH.}\n\
            {?s a test:SubClassH2.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClassH22.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClass2.?s test:hasRef ?b.?b test:hasRef ?c.?c a test:SubClassH.}=>{?s a test:SuperType.}\n\
            {?super a test:SuperType.}=>{?super a test:SuperType3.}";
        let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);

        //println!("Content {:?}", content);
        //println!("Rules {:?}", rules);


        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add_ref(rule);
        }
        let backward_head = Triple{s:VarOrTerm::newVar("?newVar".to_string(),&mut encoder),p:VarOrTerm::newTerm("a".to_string(),&mut encoder),o:VarOrTerm::newTerm("test:SuperType".to_string(),&mut encoder)};
        let var_encoded= encoder.add("?newVar".to_string());
        let result_encoded = encoder.add("<http://example2.com/a>".to_string());
        //println!("encoded {:?}", encoder.decoded);
        let mut store = TripleStore{rules:rules, rules_index , triple_index, encoder };
        //        let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());
        assert_eq!(4,store.len());

        store.compute_sprite(&backward_head);
        store.materialize();
        assert_eq!(7,store.len());

    }
    #[test]
    fn test_sprite_compute_hierarchy(){
        let timer_load = ::std::time::Instant::now();

        let mut encoder = Encoder::new();
        let size = 10000;
        let mut data  = String::new();
        for i in 0..size{
            data += &format!("<http://example2.com/a{}> a test:SubClass0.\n",i);
            data += &format!("{{?s a test:SubClass{}.}}=>{{?s a test:SubClass{}.}}\n",i,(i+1));
        }
         let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);

        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add_ref(rule);
        }
        let backward_head = Triple{s:VarOrTerm::newVar("?newVar".to_string(),&mut encoder),p:VarOrTerm::newTerm("a".to_string(),&mut encoder),o:VarOrTerm::newTerm(format!("test:SubClass{}",size),&mut encoder)};

        let mut store = TripleStore{rules:rules, rules_index , triple_index, encoder };
        let load_time = timer_load.elapsed();
        println!("Load Time: {:.2?}", load_time);
        assert_eq!(size,store.len());
        let timer_load = ::std::time::Instant::now();
        store.compute_sprite(&backward_head);
        let csprite_time = timer_load.elapsed();
        println!("CSprite Time: {:.2?}", csprite_time);
        let timer_load = ::std::time::Instant::now();
        store.materialize();
        assert_eq!(2*size,store.len());
        let load_time = timer_load.elapsed();
        println!("Materialization Time: {:.2?}", load_time);

    }

    #[test]
    fn test_rewrite_hierarchy_csprite(){
        let mut encoder = Encoder::new();
        let data="<http://example2.com/a> a test:SubClass.\n\
            {?s a test:SubClassH1.}=>{?s a test:SubClassH.}\n\
            {?s a test:SubClassH2.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClassH3.}=>{?s a test:SubClassH2.}";
        let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);
        println!("encoded {:?}", encoder.decoded);
        println!("{:?}",rules);

        let rc_rules = rules.into_iter().map(|x|Rc::new(x)).collect();
        let rewritten_rules = rewrite_hierarchy(&rc_rules);
        println!("{:?}",rewritten_rules);
    }
    // #[test]
    // fn test_eval_backward_multiple_rules(){
    //     let mut store = ReasoningStore::new();
    //     store.parse_and_add_rule("@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n \
    //     {?s rdf:type test:SubClass.}=>{?s rdf:type test:SuperType.}\n\
    //     {?s rdf:type test:SubClass2.}=>{?s rdf:type test:SuperType.}");
    //     store.load_abox( b"<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .".as_ref());
    //     store.load_abox( b"<http://example2.com/c> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass2> .".as_ref());
    //
    //     // diff variable names
    //     let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());
    //     let  bindings = store.eval_backward( &backward_head);
    //     let mut result_bindings: Binding = Binding::new();
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/a".to_string()).unwrap()));
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/c".to_string()).unwrap()));
    //
    //     assert_eq!(result_bindings, bindings);
    // }
    // #[test]
    // fn test_eval_backward_nested_rules(){
    //     let mut store = ReasoningStore::new();
    //     store.parse_and_add_rule("@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n \
    //     {?s rdf:type test:SubClass. ?s test:hasRef ?o. ?o rdf:type test:SubClass2.}=>{?s rdf:type test:SuperType.}\n\
    //     {?q rdf:type test:SubClassTemp.}=>{?q rdf:type test:SubClass2.}");
    //     store.load_abox( b"<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .".as_ref());
    //     store.load_abox( b"<http://example2.com/b> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClassTemp> .".as_ref());
    //     store.load_abox( b"<http://example2.com/a> <http://www.test.be/test#hasRef> <http://example2.com/b> .".as_ref());
    //
    //     // diff variable names
    //     let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());
    //     let  bindings = store.eval_backward( &backward_head);
    //     let mut result_bindings: Binding = Binding::new();
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/a".to_string()).unwrap()));
    //
    //     assert_eq!(result_bindings, bindings);
    // }
}
