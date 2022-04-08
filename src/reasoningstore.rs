extern crate oxigraph;

    use oxigraph::MemoryStore;
    use oxigraph::model::*;
    use oxigraph::sparql::{QueryResults, Query};
    use oxigraph::io::{GraphFormat, DatasetFormat};
    use std::collections::HashMap;
    use oxigraph::model::NamedOrBlankNode;
    use rio_turtle::{TurtleParser, TurtleError};
    use rio_api::parser::TriplesParser;
    use rio_api::model::NamedNode as RioNode;
    use std::io::BufRead;
    use std::io;


#[derive(Debug,  Clone)]
    pub struct ReasonerTriple{
        pub s: NamedOrBlankNode,
        pub p: NamedOrBlankNode,
        pub o: NamedOrBlankNode
    }
    impl ReasonerTriple{
        pub fn new(s: String, p: String, o: String) -> ReasonerTriple{
            ReasonerTriple{s:ReasonerTriple::convert(s),p:ReasonerTriple::convert(p), o:ReasonerTriple::convert(o)}
        }
        fn convert(iri: String) -> NamedOrBlankNode{
            let result : NamedOrBlankNode;
            if(iri.starts_with('?')){
                let var_name = &iri[1..];
                result = NamedOrBlankNode::from(BlankNode::new(var_name).unwrap());
            }else{
                result = NamedOrBlankNode::from(NamedNode::new(iri).unwrap());
            }
            result
        }
    }
    #[derive(Debug,  Clone)]
    pub struct Rule{
        pub body: Vec<ReasonerTriple>,
        pub head: ReasonerTriple
    }
    pub struct RuleIndex {
        spo:Vec<Rule>,
        s:HashMap<String,  Vec::<Rule>>,
        p:HashMap<String, Vec::<Rule>>,
        o:HashMap<String,  Vec::<Rule>>,
        sp:HashMap<String,  Vec::<Rule>>,
        po:HashMap<String,  Vec::<Rule>>,
        so:HashMap<String,  Vec::<Rule>>,
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
                    if !self.s.contains_key(&s.to_string()){
                        self.s.insert(s.to_string(),Vec::new());
                    }
                    if let Some(mut rules) = self.s.get_mut(&s.to_string()){
                        rules.push(rule.clone());
                    }
                    // self.s.get(&s.to_string()).unwrap().push(rule.clone());
                }
                //p match
                if s.is_blank_node() && p.is_named_node() && o.is_blank_node(){
                    if !self.p.contains_key(&p.to_string()){
                        self.p.insert(p.to_string(),Vec::new());
                    }
                    self.p.get_mut(&p.to_string()).unwrap().push(rule.clone());
                }
                //o match
                if s.is_blank_node() && p.is_blank_node() && o.is_named_node(){
                    if !self.o.contains_key(&o.to_string()){
                        self.o.insert(o.to_string(),Vec::new());
                    }
                    self.o.get_mut(&o.to_string()).unwrap().push(rule.clone());
                }
                //sp
                if s.is_named_node() && p.is_named_node() && o.is_blank_node(){
                    let sp_str = format!("{}{}",s.to_string(),p.to_string());
                    if !self.sp.contains_key(&sp_str){
                        self.sp.insert(sp_str.clone(),Vec::new());
                    }
                    self.sp.get_mut(&sp_str).unwrap().push(rule.clone());
                }
                //so
                if s.is_named_node() && p.is_blank_node() && o.is_named_node(){
                    let so_str = format!("{}{}",s.to_string(),o.to_string());
                    if !self.so.contains_key(&so_str){
                        self.so.insert(so_str.clone(),Vec::new());
                    }
                    self.so.get_mut(&so_str).unwrap().push(rule.clone());
                }
                //po
                if s.is_blank_node() && p.is_named_node() && o.is_named_node(){
                    let po_str = format!("{}{}",p.to_string(),o.to_string());
                    if !self.po.contains_key(&po_str){
                        self.po.insert(po_str.clone(),Vec::new());
                    }
                    self.po.get_mut(&po_str).unwrap().push(rule.clone());
                }
                //spo
                if s.is_blank_node() && p.is_blank_node() && o.is_blank_node() {
                    self.spo.push(rule.clone());
                }

            }
        }

        pub fn find_match(&self, quad: &Quad)->Vec<&Rule>{
            let mut matched_triples: Vec<&Rule> = Vec::new();
            //check s
            if let Some(rule) = self.s.get(&quad.subject.to_string()){
                rule.iter().for_each(|r|matched_triples.push(r));
            }
            //check p
            if let Some(rule) = self.p.get(&quad.predicate.to_string()){
                rule.iter().for_each(|r|matched_triples.push(r));
            }
            //check o
            if let Some(rule) = self.o.get(&quad.predicate.to_string()){
                rule.iter().for_each(|r|matched_triples.push(r));
            }
            //check so
            if let Some(rule) = self.so.get(&format!("{}{}",quad.subject.to_string(),quad.object.to_string())){
                rule.iter().for_each(|r|matched_triples.push(r));
            }
            //check po
            if let Some(rule) = self.po.get(&format!("{}{}",quad.predicate.to_string(),quad.object.to_string())){
                rule.iter().for_each(|r|matched_triples.push(r));
            }
            //check sp
            if let Some(rule) = self.sp.get(&format!("{}{}",quad.subject.to_string(),quad.predicate.to_string())){
                rule.iter().for_each(|r|matched_triples.push(r));
            }
            self.spo.iter().for_each(|r| matched_triples.push(r));

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
        pub(crate) reasoning_store: MemoryStore,
        rules: Vec<Rule>,
        rules_index: RuleIndex
    }
impl ReasoningStore {
    pub fn new() -> ReasoningStore{
        ReasoningStore{store: MemoryStore::new(), reasoning_store: MemoryStore::new(),
            rules:Vec::new(), rules_index: RuleIndex::new()}
    }
    pub fn load_abox(&self, reader: impl BufRead)-> Result<(), io::Error>{
        self.store.load_dataset(reader,DatasetFormat::TriG, None)
    }
    pub fn load_tbox(&mut self, reader: impl BufRead){
        let parseStore = MemoryStore::new();
        let rdf_subClass = String::from("<http://www.w3.org/2000/01/rdf-schema#subClassOf>");
        TurtleParser::new(reader, None).parse_all(&mut |triple| {

            if triple.predicate.to_string().eq(&rdf_subClass){
                let str_len = triple.object.to_string().len();
                let object_str =  &triple.object.to_string()[1..str_len-1];
                let str_len = triple.subject.to_string().len();
                let subject_str =  &triple.subject.to_string()[1..str_len-1];
                if let Ok(named_subject) = NamedNode::new(subject_str){
                    let body = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap()), o:  NamedOrBlankNode::from(named_subject)};
                    if let Ok(named) = NamedNode::new(object_str) {
                        let head = ReasonerTriple { s: NamedOrBlankNode::from(BlankNode::new("s").unwrap()), p: NamedOrBlankNode::from(NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap()), o: NamedOrBlankNode::from(named) };
                        let mut body_rules = Vec::new();
                        body_rules.push(body);
                        let rule = Rule { body: body_rules, head: head };
                        self.add_rule(rule.clone());
                    }
                }
            }
            Ok(()) as Result<(), TurtleError>
        }).unwrap();

    }
    pub fn len_rules(&self) -> usize{
        self.rules_index.spo.len()
    }
    pub fn len_abox(&self) -> usize{
        self.store.len()
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
