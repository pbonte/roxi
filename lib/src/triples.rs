use crate::Encoder;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum VarOrTerm {
    Var(Variable),
    Term(TermImpl),
    // Literal(Literal),
    // BlankNode(BlankNode)
}
impl VarOrTerm {
    pub fn new_term(iri: String) -> VarOrTerm {
        let encoded = Encoder::add(iri);
        VarOrTerm::Term(TermImpl { iri: encoded })
    }
    pub fn new_var(name: String) -> VarOrTerm {
        let encoded = Encoder::add(name);
        VarOrTerm::Var(Variable { name: encoded })
    }
    pub fn new_encoded_term(iri: usize) -> VarOrTerm {
        VarOrTerm::Term(TermImpl { iri })
    }
    pub fn new_encoded_var(name: usize) -> VarOrTerm {
        VarOrTerm::Var(Variable { name })
    }
    pub(crate) fn as_term(&self) -> &TermImpl {
        if let VarOrTerm::Term(t) = self {
            t
        } else {
            panic!("Not a term")
        }
    }
    pub(crate) fn as_var(&self) -> &Variable {
        if let VarOrTerm::Var(v) = self {
            v
        } else {
            panic!("Not a Var")
        }
    }
    pub fn is_var(&self) -> bool {
        match self {
            Self::Var(_) => true,
            Self::Term(_) => false,
        }
    }
    pub fn is_term(&self) -> bool {
        !self.is_var()
    }
    pub fn to_encoded(&self) -> usize {
        match self {
            Self::Var(var) => var.name,
            Self::Term(term) => term.iri,
        }
    }
    fn rem_first_and_last(value: &str) -> &str {
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        chars.as_str()
    }
    pub fn convert(var_or_term: String) -> VarOrTerm {
        if var_or_term.starts_with('?') {
            let var_name = &var_or_term[1..];
            VarOrTerm::new_var(var_name.to_string())
        } else {
            let mut iri_prefix = var_or_term;
            if !iri_prefix.starts_with('<') {
                iri_prefix = format!("<{}>", iri_prefix).to_string();
            }
            VarOrTerm::new_term(iri_prefix)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Variable {
    pub(crate) name: usize,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TermImpl {
    pub(crate) iri: usize,
}
// #[derive(Debug, Clone, Eq, PartialEq, Hash)]
// pub struct Literal{
//     pub value: String
// }
// #[derive(Debug, Clone, Eq, PartialEq, Hash)]
// pub struct BlankNode{
//     pub id: String
// }
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Triple {
    pub s: VarOrTerm,
    pub p: VarOrTerm,
    pub o: VarOrTerm,
    pub g: Option<VarOrTerm>
}

impl Triple {
    pub fn from(
        subject: String,
        property: String,
        object: String
    ) -> Triple {
        Triple {
            s: VarOrTerm::convert(subject),
            p: VarOrTerm::convert(property),
            o: VarOrTerm::convert(object),
            g: None
        }
    }
    pub fn from_with_graph_name(
        subject: String,
        property: String,
        object: String,
        graph_name:String,
    ) -> Triple {
        let mut triple = Self::from(subject, property, object);
        triple.g = Some(VarOrTerm::convert(graph_name));
        triple
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rule {
    pub body: Vec<Triple>,
    pub head: Triple,
}
