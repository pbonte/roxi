use oxigraph::model::{BlankNode, NamedNode, NamedOrBlankNode};


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
        if iri.starts_with('?'){
            let var_name = &iri[1..];
            result = NamedOrBlankNode::from(BlankNode::new(var_name).unwrap());
        }else{
            result = NamedOrBlankNode::from(NamedNode::new(iri).unwrap());
        }
        result
    }
    pub fn to_string(&self) -> String{
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