extern crate core;
pub mod ruleindex;
pub mod tripleindex;
pub mod imars;
pub mod imars_reasoner;
pub mod bindings;
pub mod triples;
pub mod encoding;
pub mod queryengine;
pub mod reasoner;
use crate::ruleindex::RuleIndex;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::tripleindex::TripleIndex;
use std::fmt::Write;
use crate::bindings::Binding;

#[cfg(not(test))]
use log::{info, warn,trace}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn, println as trace};
use crate::encoding::Encoder;
use crate::queryengine::{QueryEngine, SimpleQueryEngine};
use crate::reasoner::Reasoner;
use crate::triples::{Rule, TermImpl, Triple, VarOrTerm}; // Workaround to use prinltn! for logs.



pub struct TripleStore{
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index : TripleIndex,
    pub encoder: Encoder,
    reasoner: Reasoner
}



impl TripleStore {
    pub fn new() -> TripleStore{
        TripleStore{rules: Vec::new(), rules_index: RuleIndex::new(), triple_index: TripleIndex::new(), encoder: Encoder::new(), reasoner: Reasoner{ } }
    }
    pub fn from(data:&str) -> TripleStore{
        let mut encoder = Encoder::new();
        let (mut content, mut rules) = TripleStore::parse(data.to_string(),&mut encoder);
        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter(){
            rules_index.add_ref(rule);
        }
        TripleStore{rules:rules, rules_index , triple_index, encoder,reasoner: Reasoner{ } }
    }
    pub fn add(&mut self, triple: Triple){
        trace!{"Adding triple: {:?}", self.decode_triple(&triple) }
        self.triple_index.add(triple);
    }
    pub fn add_ref(&mut self, triple: Rc<Triple>){
        trace!{"Adding triple: {:?}", self.decode_triple(triple.as_ref()) }
        self.triple_index.add_ref(triple);
    }
    pub fn remove_ref(&mut self, triple: Rc<Triple>){
        trace!{"Removing triple: {:?}", self.decode_triple(triple.as_ref()) }
        self.triple_index.remove_ref(triple);
    }
    pub(crate) fn add_rules(&mut self, rules: Vec<Rule>) {
        rules.into_iter().for_each(|rule|self.rules_index.add(rule));
    }
    pub fn len(&self) -> usize{
        self.triple_index.len()
    }
    fn decode_triple(&self, triple:  &Triple) -> String {
        let s = self.encoder.decode(&triple.s.to_encoded()).unwrap();
        let p = self.encoder.decode(&triple.p.to_encoded()).unwrap();
        let o = self.encoder.decode(&triple.o.to_encoded()).unwrap();
        format!("{} {} {}",s,p,o)
    }
    pub fn materialize(&mut self) -> Vec<Triple>{
        self.reasoner.materialize(&mut self.triple_index,&self.rules_index)
    }

    //Backward chaining
    fn eval_backward(&self, rule_head: &Triple)->Binding{
        let sub_rules : Vec<(Rc<Rule>, Vec<(usize, usize)>)> = self.find_subrules(rule_head);
        let mut all_bindings = Binding::new();
        for (sub_rule,var_subs) in sub_rules.into_iter(){
            let mut rule_bindings = Binding::new();
            for rule_atom in &sub_rule.body{
                if let Some(result_bindings) = self.triple_index.query(rule_atom,None) {
                    rule_bindings = rule_bindings.join(&result_bindings);
                    //recursive call
                    let recursive_bindings = self.eval_backward(rule_atom);
                    rule_bindings.combine(recursive_bindings);
                }
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
    use crate::{Rule, Triple, TripleStore, TermImpl, VarOrTerm, RuleIndex, TripleIndex, Encoder, rewrite_hierarchy, SimpleQueryEngine, QueryEngine};
    use crate::reasoner::Reasoner;

    #[test]
    fn test_parse(){
        let mut encoder = Encoder::new();
        let data=":a a :C0.\n\
            {?a a :C0}=>{?a a :C1}\n\
            {?a a :C1}=>{?a a :C2}\n\
            {?a a :C2}=>{?a a :C3}";

        let mut store = TripleStore::from(data);

        let mat = store.materialize();
        println!("Length: {:?}", store.len());
        println!("Length Mat: {:?}", mat.len());
    }


    #[test]
    fn test_store() {
        let timer = ::std::time::Instant::now();
        let mut rules = Vec::new();
        let mut encoder = Encoder::new();
        let max_depth = 5;
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

        let mut store = TripleStore{rules:Vec::new(), rules_index , triple_index, encoder,reasoner: Reasoner{ } };

        store.materialize();
        let elapsed = timer.elapsed();

        let result = SimpleQueryEngine::query(&store.triple_index, &Vec::from([query]), None);

        println!("Processed in: {:.2?}", elapsed);
        println!("Result: {:?}", result);

    }

    #[test]
    fn test_eval_backward_rule(){
        let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClass.\n\
            {?s a test:SubClass.}=>{?s a test:SubClass2.}\n
            {?s a test:SubClass2.?s test:hasRef ?b.?b test:hasRef ?c.?c a test:SubClass2.}=>{?s a test:SuperType.}";
        let mut store = TripleStore::from(data);
        let encoder = &mut store.encoder;
        let backward_head = Triple{s:VarOrTerm::newVar("?newVar".to_string(), encoder),p:VarOrTerm::newTerm("a".to_string(), encoder),o:VarOrTerm::newTerm("test:SuperType".to_string(), encoder)};
        let var_encoded= encoder.add("?newVar".to_string());
        let result_encoded = encoder.add("<http://example2.com/a>".to_string());

        let  bindings = store.eval_backward( &backward_head);
        let result_bindings = HashMap::from([
            (var_encoded, Vec::from([result_encoded]))
        ]);
        assert_eq!(result_bindings.get(&12), bindings.get(&12));
    }
    #[test]
    fn test_incomplete_rule_match(){
        let data=":a in :b.\n\
            {?a in ?b. ?b in ?c}=>{?a in ?c.}";

        let mut store = TripleStore::from(data);
        assert_eq!(1,store.len());
        store.materialize();
        assert_eq!(1,store.len());

    }
    #[test]
    fn test_no_var_query(){
        let data=":a in :b.\n\
            {:a in :b}=>{:a in :c}";

        let mut store = TripleStore::from(data);
        assert_eq!(1,store.len());
        store.materialize();
        assert_eq!(2,store.len());

    }
    #[test]
    fn test_sprite_compute(){
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
        let mut store = TripleStore::from(data);

        let mut encoder = &mut store.encoder;
        let backward_head = Triple { s: VarOrTerm::newVar("?newVar".to_string(), encoder), p: VarOrTerm::newTerm("a".to_string(), encoder), o: VarOrTerm::newTerm("test:SuperType".to_string(),  encoder) };
        let var_encoded = encoder.add("?newVar".to_string());
        let result_encoded = encoder.add("<http://example2.com/a>".to_string());

        assert_eq!(4,store.len());

        store.compute_sprite(&backward_head);
        store.materialize();
        assert_eq!(7,store.len());

    }
    //todo move to benchmark
    #[test]
    fn test_sprite_compute_hierarchy(){
        let timer_load = ::std::time::Instant::now();

        let size = 10;
        let mut data  = String::new();
        for i in 0..size{
            data += &format!("<http://example2.com/a{}> a test:SubClass0.\n",i);
            data += &format!("{{?s a test:SubClass{}.}}=>{{?s a test:SubClass{}.}}\n",i,(i+1));
        }
        let mut store = TripleStore::from(data.as_str());

        let backward_head = Triple{s:VarOrTerm::newVar("?newVar".to_string(),&mut store.encoder),p:VarOrTerm::newTerm("a".to_string(),&mut store.encoder),o:VarOrTerm::newTerm(format!("test:SubClass{}",size),&mut store.encoder)};

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
