use crate::reasoningstore::triple::ReasonerTriple;

#[derive(Debug,  Clone)]
pub struct Rule{
    pub body: Vec<ReasonerTriple>,
    pub head: ReasonerTriple
}