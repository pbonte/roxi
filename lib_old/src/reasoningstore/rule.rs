use crate::reasoningstore::triple::ReasonerTriple;

#[derive(Debug,  Clone,Eq, PartialEq)]
pub struct Rule{
    pub body: Vec<ReasonerTriple>,
    pub head: ReasonerTriple
}

impl Rule{
    pub fn to_string(&self) -> String{
        let mut final_string: String = "{".to_owned();
        self.body.iter().for_each(|b| final_string.push_str(b.to_string().as_str()));
        final_string.push_str("}=>{");
        final_string.push_str(self.head.to_string().as_str());
        final_string.push_str("}.");
        final_string
    }
}