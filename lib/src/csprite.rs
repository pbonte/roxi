use std::collections::HashSet;
use std::rc::Rc;
use crate::{BackwardChainer, Encoder, Reasoner, Rule, RuleIndex, Triple, TripleIndex, TripleStore};
#[cfg(not(test))]
use log::{info, warn,trace}; // Use log crate when building application
use std::fmt::Write;

#[cfg(test)]
use std::{println as info, println as warn, println as trace};
use std::cell::RefCell;
use crate::imars_window::{ImarsWindow, WindowConsumer};
use crate::reasoner::CSpriteReasoner;

pub struct CSprite{
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index : TripleIndex,
    pub encoder: Encoder,
    window_reasoner: CSpriteReasoner,
    reasoner: Reasoner,
    imars: ImarsWindow<Triple>
}

impl CSprite{
    pub fn new() -> CSprite{
        CSprite{rules: Vec::new(), rules_index: RuleIndex::new(), triple_index: TripleIndex::new(), encoder: Encoder::new(), window_reasoner: CSpriteReasoner{ } , reasoner: Reasoner{} , imars: ImarsWindow::new_no_window()  }

    }
    pub fn from(data:&str) -> CSprite{
        let triple_store = TripleStore::from(&data);
        CSprite{rules: triple_store.rules, rules_index: triple_store.rules_index , triple_index: triple_store.triple_index, encoder: triple_store.encoder, window_reasoner: CSpriteReasoner{ }, reasoner: Reasoner{},imars: ImarsWindow::new_no_window()  }
    }
    pub fn window_update(&mut self, new_data: Vec<(i32, Rc<Triple>)>, old_data: Vec<(i32, Rc<Triple>)>, last_ts:&i32){
        println!("New data: {:?}",Self::decode_triples(&new_data,&self.encoder));
        println!("Old data: {:?}",Self::decode_triples(&old_data,&self.encoder));

        //remove expired data
        let old_items = self.imars.remove_old_elements(last_ts);
        println!("Deleting expired: {:?}",Self::decode_triples(&old_items,&self.encoder));

        old_items.into_iter().for_each(|(_ts,item)|self.triple_index.remove_ref(&item));

        //add new data

        new_data.iter().for_each(|(ts, triple)|{
            self.imars.add_without_update(triple.clone(),*ts);
            self.add_ref(triple.clone());
        });
        let materialization = self.window_reasoner.materialize(&new_data,&mut self.triple_index,&self.rules_index,&mut self.imars,&self.encoder);
        println!("inferred data: {:?}",Self::decode_triples(&materialization,&self.encoder));

        //add materialization to maintenance program
        //materialization.into_iter().for_each(|(ts,t)|self.imars.add_without_update(t,ts));
    }
    fn decode_triples(triples: &Vec<(i32,Rc<Triple>)>, encoder: &Encoder) -> String {
        let mut res = String::new();
        for (ts,triple) in triples {
            let decoded_s = encoder.decode(&triple.s.to_encoded()).unwrap();
            let decoded_p = encoder.decode(&triple.p.to_encoded()).unwrap();
            let decoded_o = encoder.decode(&triple.o.to_encoded()).unwrap();

            write!(&mut res, "{} {} {} @ {}.\n", decoded_s, decoded_p, decoded_o,ts).unwrap();
        }
        res
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
        self.triple_index.remove_ref(&triple);
    }
    pub fn add_rules(&mut self, rules: Vec<Rule>) {
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
    pub fn materialize_window(&mut self, window: Rc<RefCell<ImarsWindow<Triple>>>) -> Vec<(i32, Triple)>{
        //self.window_reasoner.materialize(&mut self.triple_index, &self.rules_index, &self.imars)
        Vec::new()
    }
    pub fn materialize(&mut self) -> Vec<Triple>{
        self.reasoner.materialize(&mut self.triple_index, &self.rules_index)
    }
    pub fn clear(&mut self){
        self.triple_index.clear();

    }
    pub(crate) fn compute_sprite(&mut self, query: &Triple) {
        let  (backward_rules, hierarcies) = self.eval_backward_csprite( query);

        // new rules
        let mut new_rules: Vec<Rc<Rule>> = backward_rules.into_iter().filter(|r|r.body.len()>1).collect();
        for hierarchy in hierarcies{
            let rewritten_hierarchy = Self::rewrite_hierarchy(&hierarchy);
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
         let mut matched_rules = HashSet::new();
         let mut hierarchies = Vec::new();
         self.eval_backward_csprite_helper(rule_head,&mut matched_rules,false, &mut hierarchies);
         (matched_rules, hierarchies)
        //self.eval_backward_csprite_helper_with_stack(rule_head)
    }
    fn eval_backward_csprite_helper(&self, rule_head: &Triple, matched_rules: &mut HashSet<Rc<Rule>>, hierarchy:bool, hierarchies: &mut Vec<Vec<Rc<Rule>>>){
        //TODO check cycles
        let sub_rules : Vec<(Rc<Rule>, Vec<(usize, usize)>)> = BackwardChainer::find_subrules(&self.rules_index,rule_head);
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
            let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> = BackwardChainer::find_subrules(&self.rules_index,&current_head);
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
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::{Rule, Triple, TripleStore, TermImpl, VarOrTerm, RuleIndex, TripleIndex, Encoder, SimpleQueryEngine, QueryEngine, Parser, BackwardChainer};
    use crate::csprite::CSprite;
    use crate::reasoner::Reasoner;

    #[test]
    fn test_sprite_compute(){
        let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClassH1.\n\
            {?s a test:SubClass.}=>{?s a test:SubClass2.}\n\
            {?s a test:SubClass2.}=>{?s a test:SubClass.}\n\
            {?s a test:SubClass0.}=>{?s a test:SubClass2.}\n\
            {?s a test:SubClass01.}=>{?s a test:SubClass0.}\n\
            {?s a test:SubClassH1.}=>{?s a test:SubClassH.}\n\
            {?s a test:SubClassH2.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClassH22.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClass2.?s test:hasRef ?b.?b test:hasRef ?c.?c a test:SubClassH.}=>{?s a test:SuperType.}\n\
            {?super a test:SuperType.}=>{?super a test:SuperType3.}";
        let mut store = CSprite::from(data);

        let encoder = &mut store.encoder;
        let backward_head = Triple { s: VarOrTerm::new_var("?newVar".to_string(), encoder), p: VarOrTerm::new_term("a".to_string(), encoder), o: VarOrTerm::new_term("test:SuperType".to_string(), encoder), g:None };


        //assert_eq!(4,store.len());
        let validation_triple = Triple { s: VarOrTerm::new_term("<http://example2.com/a>".to_string(), encoder), p: VarOrTerm::new_term("a".to_string(), encoder), o: VarOrTerm::new_term("test:SuperType".to_string(), encoder), g: None };

        store.compute_sprite(&backward_head);
        store.materialize();
        assert_eq!(true, store.triple_index.contains(&validation_triple));
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
        let mut store = CSprite::from(data.as_str());

        let backward_head = Triple{s:VarOrTerm::new_var("?newVar".to_string(), &mut store.encoder),p:VarOrTerm::new_term("a".to_string(), &mut store.encoder),o:VarOrTerm::new_term(format!("test:SubClass{}", size), &mut store.encoder), g: None};

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
        let ( _content, rules) = Parser::parse(data.to_string(),&mut encoder);
        println!("{:?}",rules);

        let rc_rules = rules.into_iter().map(|x|Rc::new(x)).collect();
        let rewritten_rules = CSprite::rewrite_hierarchy(&rc_rules);
        println!("{:?}",rewritten_rules);
    }
    // #[test]
    // fn test_transitive(){
    //     let rules ="{?a in ?b.?b in ?c}=>{?a in ?c}";
    //     let data =":1 in :0.\n\
    //         :2 in :1.\n\
    //         :3 in :2.\n\
    //         :4 in :3.\n\
    //         :5 in :4.\n\
    //         :6 in :5";
    //     let csprite = CSprite::from_with_window(rules, 4, 2);
    //     let (mut content, mut rules) = Parser::parse(data.to_string(), &mut csprite.borrow_mut().encoder);
    //
    //
    //
    //
    //
    //     content.into_iter().enumerate().for_each(|(i, t)| csprite.borrow_mut().window.add(t, i as i32));
    //
    //     //contains 4 triples and 1 inferred triple
    //     assert_eq!(19, csprite.borrow_mut().window.len());
    // }
}