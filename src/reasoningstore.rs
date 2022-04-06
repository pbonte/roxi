extern crate oxigraph;

    use oxigraph::MemoryStore;
    use oxigraph::model::*;
    use oxigraph::sparql::{QueryResults, Query};
    use oxigraph::io::GraphFormat;
    use std::collections::HashMap;
    use oxigraph::model::NamedOrBlankNode;


    #[derive(Debug,  Clone)]
    pub struct ReasonerTriple{
        pub s: NamedOrBlankNode,
        pub p: NamedOrBlankNode,
        pub o: NamedOrBlankNode
    }
    #[derive(Debug,  Clone)]
    pub struct Rule{
        pub body: Vec<ReasonerTriple>,
        pub head: ReasonerTriple
    }
    pub struct RuleIndex {
        spo:Vec<Rule>,
        s:HashMap<String,  Rule>,
        p:HashMap<String, Rule>,
        o:HashMap<String,  Rule>,
        sp:HashMap<String,  Rule>,
        po:HashMap<String,  Rule>,
        so:HashMap<String,  Rule>,
    }
    impl  RuleIndex {
        pub fn new() -> RuleIndex{
            RuleIndex{s:HashMap::new(),
                p:HashMap::new(),
                o:HashMap::new(),
                so:HashMap::new(),
                po:HashMap::new(),
                sp:HashMap::new(),
                spo:Vec::new()}
        }
        pub fn add(&mut self, rule:  Rule ){
            for ReasonerTriple{s ,p,o}  in rule.body.iter(){
                //s match
                if s.is_named_node() && p.is_blank_node() && o.is_blank_node(){
                    self.s.insert(s.to_string(),rule.clone());
                }
                //p match
                if s.is_blank_node() && p.is_named_node() && o.is_blank_node(){
                    self.p.insert(p.to_string(),rule.clone());
                }
                //o match
                if s.is_blank_node() && p.is_blank_node() && o.is_named_node(){
                    self.o.insert(s.to_string(),rule.clone());
                }
                //sp
                if s.is_named_node() && p.is_named_node() && o.is_blank_node(){
                    self.sp.insert(format!("{}{}",s.to_string(),p.to_string()),rule.clone());
                }
                //so
                if s.is_named_node() && p.is_blank_node() && o.is_named_node(){
                    self.so.insert(format!("{}{}",s.to_string(),o.to_string()),rule.clone());
                }
                //po
                if s.is_blank_node() && p.is_named_node() && o.is_named_node(){
                    self.po.insert(format!("{}{}",p.to_string(),o.to_string()),rule.clone());
                }
                //spo
                self.spo.push(rule.clone());

            }
        }
        pub fn find_match(&self, quad: &Quad)->Vec<&Rule>{
            let mut matched_triples: Vec<&Rule> = Vec::new();
            //check s
            if let Some(rule) = self.s.get(&quad.subject.to_string()){
                matched_triples.push(rule);
            }
            //check p
            if let Some(rule) = self.p.get(&quad.predicate.to_string()){
                matched_triples.push(rule);
            }
            //check o
            if let Some(rule) = self.o.get(&quad.predicate.to_string()){
                matched_triples.push(rule);
            }
            //check so
            if let Some(rule) = self.so.get(&format!("{}{}",quad.subject.to_string(),quad.object.to_string())){
                matched_triples.push(rule);
            }
            //check po
            if let Some(rule) = self.po.get(&format!("{}{}",quad.predicate.to_string(),quad.object.to_string())){
                matched_triples.push(rule);
            }
            //check sp
            if let Some(rule) = self.sp.get(&format!("{}{}",quad.subject.to_string(),quad.predicate.to_string())){
                matched_triples.push(rule);
            }
            //self.spo.iter().for_each(|r| matched_triples.push(r));

            matched_triples
        }
    }
    impl ReasonerTriple{
        fn to_string(&self) -> String{
            let mut final_string: String = "".to_owned();
            match &self.s{
                NamedOrBlankNode::NamedNode(node_iri) => final_string.push_str(&format!(" {} ", node_iri.to_string())),
                NamedOrBlankNode::BlankNode(var_name) => final_string.push_str(&format!(" ?{} ", var_name.as_str())),
            }
            match &self.p{
                NamedOrBlankNode::NamedNode(node_iri) => final_string.push_str(&format!(" {} ", node_iri.to_string())),
                NamedOrBlankNode::BlankNode(var_name) => final_string.push_str(&format!(" ?{} ", var_name.as_str())),
            }
            match &self.o{
                NamedOrBlankNode::NamedNode(node_iri) => final_string.push_str(&format!(" {} .", node_iri.to_string())),
                NamedOrBlankNode::BlankNode(var_name) => final_string.push_str(&format!(" ?{} .", var_name.as_str())),
            }
            final_string
        }
    }
    fn find_rule_match<'a>(quad:&'a Quad, rules: &'a Vec<Rule>) -> Vec<&'a Rule>{
        let mut matched_triples: Vec<&Rule> = Vec::new();
        for rule in rules.iter(){
            for ReasonerTriple{s ,p,o}  in rule.body.iter(){
                let mut match_triple = true;

                match s{
                    NamedOrBlankNode::NamedNode(node_iri) if !quad.subject.to_string().eq(&node_iri.to_string())=> match_triple=false,
                    _ => match_triple = true,
                }
                match p{
                    NamedOrBlankNode::NamedNode(node_iri) if !quad.predicate.to_string().eq(&node_iri.to_string())=> match_triple=false,
                    _ => match_triple = true,
                }
                match o{
                    NamedOrBlankNode::NamedNode(node_iri) if !quad.object.to_string().eq(&node_iri.to_string())=>  match_triple=false,
                    _ => match_triple = true,
                }
                if match_triple {
                    matched_triples.push(rule);
                }
            }
        }
        matched_triples
    }
    pub fn convert_to_query(rule : &Rule) -> Query{
        let body_string:String = rule.body.iter().map(|r| r.to_string()).collect();
        let query_string = format!("CONSTRUCT {{ {} }} WHERE {{ {} }}",rule.head.to_string(), body_string);
        Query::parse(&query_string,None).unwrap()
    }

    pub struct ReasoningStore{
        pub store: MemoryStore,
        reasoning_store: MemoryStore,
        rules: Vec<Rule>,
        rules_index: RuleIndex
    }
impl ReasoningStore {
    pub fn new() -> ReasoningStore{
        ReasoningStore{store: MemoryStore::new(), reasoning_store: MemoryStore::new(),
            rules:Vec::new(), rules_index: RuleIndex::new()}
    }
    pub fn add_rule(&mut self,rule:Rule){
        self.rules.push(rule.clone());
        self.rules_index.add(rule.clone());
    }
    pub fn materialize(&self) {
        let mut tripe_queue: Vec<Quad> = self.store.iter().collect();
        // iterate over all triples
        let mut queue_iter = 0;
        while queue_iter < tripe_queue.len() {
            let mut temp_triples = Vec::new();
            let quad = tripe_queue.get(queue_iter).unwrap();
            if !self.reasoning_store.contains(quad) {
                self.reasoning_store.insert(quad.clone());

                //let matched_rules = find_rule_match(&quad, &rules); // without indexing
                let matched_rules = self.rules_index.find_match(&quad);
                // find matching rules
                for matched_rule in matched_rules.into_iter() {
                    let q = convert_to_query(matched_rule);
                    if let QueryResults::Graph(solutions) = self.reasoning_store.query(q).unwrap() {
                        for sol in solutions.into_iter() {
                            match sol {
                                Ok(s) => temp_triples.push(Quad::new(s.subject.clone(), s.predicate.clone(), s.object.clone(), None)),
                                _ => (),
                            }
                        }
                    }
                }
            }
            queue_iter += 1;
            temp_triples.iter().for_each(|t| tripe_queue.push(t.clone()));
        }
    }
}
