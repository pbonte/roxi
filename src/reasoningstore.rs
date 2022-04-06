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
        // use oxigraph::model::NamedNode;
// use oxigraph::sparql::Query;
//
// let query_str = "SELECT ?s ?p ?o WHERE { ?s ?p ?o . }";
// let mut query = Query::parse(query_str, None)?;
        let body_string:String = rule.body.iter().map(|r| r.to_string()).collect();
        let query_string = format!("CONSTRUCT {{ {} }} WHERE {{ {} }}",rule.head.to_string(), body_string);
        Query::parse(&query_string,None).unwrap()
    }
