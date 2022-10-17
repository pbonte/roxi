use spargebra::Query;
use crate::Syntax;

pub trait R2ROperator<I,O>: Send{
    fn load_triples(&mut self, data: &str, syntax: Syntax) -> Result<(),&'static str>;
    fn load_rules(&mut self, data: &str) -> Result<(),&'static str>;
    fn add(&mut self, data: I);
    fn remove(&mut self, data: &I);
    fn materialize(&mut self);
    fn execute_query(&self,query: &Query) -> Vec<O>;
}

