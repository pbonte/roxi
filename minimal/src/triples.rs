use crate::Encoder;

#[derive(Debug,  Clone, Eq, PartialEq, Hash)]
pub enum VarOrTerm{
    Var(Variable),
    Term(TermImpl)
}
impl VarOrTerm{
    pub fn new_term(iri:String, encoder: &mut Encoder) -> VarOrTerm{
        let  encoded = encoder.add(iri);
        VarOrTerm::Term(TermImpl{iri:encoded})
    }
    pub  fn new_var(name:String, encoder: &mut Encoder) -> VarOrTerm{
        let encoded = encoder.add(name);
        VarOrTerm::Var(Variable{name:encoded})
    }
    pub fn new_encoded_term(iri:usize) -> VarOrTerm{
        VarOrTerm::Term(TermImpl{iri})
    }
    pub  fn new_encoded_var(name:usize) -> VarOrTerm{
        VarOrTerm::Var(Variable{name})
    }
    pub(crate) fn as_term(&self) -> &TermImpl{
        if let VarOrTerm::Term(t) = self {t} else{ panic!("Not a term")}
    }
    pub(crate) fn as_var(&self) -> &Variable{
        if let VarOrTerm::Var(v) = self {v} else{ panic!("Not a Var")}
    }
    pub(crate) fn is_var(&self) -> bool{
        match self {
            Self::Var(_) => true,
            Self::Term(_) => false,
        }}
    pub(crate) fn is_term(&self) -> bool {
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
    pub(crate) name: usize
}
#[derive(Debug,  Clone, Eq, PartialEq, Hash)]
pub struct TermImpl {
    pub(crate) iri: usize
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