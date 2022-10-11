use std::collections::HashSet;
use std::mem;
use crate::sparql::Binding;

pub enum StreamOperator{
    RSTREAM, ISTREAM, DSTREAM
}
pub struct Relation2StreamOperator {
    stream_operator: StreamOperator,
    old_result: HashSet<Vec<Binding>>,
    new_result: HashSet<Vec<Binding>>,
    ts: usize
}

impl Relation2StreamOperator {
    pub fn new(stream_operator: StreamOperator, start_time: usize) -> Relation2StreamOperator {
        Relation2StreamOperator {stream_operator, old_result: HashSet::new(), new_result: HashSet::new(),ts: start_time}
    }
    pub fn eval(&mut self, new_repsonse: Vec<Vec<Binding>>, ts: usize) -> Vec<Vec<Binding>>{
        match self.stream_operator {
            StreamOperator::RSTREAM => new_repsonse,
            StreamOperator::ISTREAM => {
                let to_compare = new_repsonse.clone();
                self.prepare_compare(new_repsonse, ts);
                to_compare.into_iter().filter(|b| !self.old_result.contains(b)).collect()
            },
            StreamOperator::DSTREAM => {
                self.prepare_compare(new_repsonse, ts);
                let to_compare = self.old_result.clone();
                to_compare.into_iter().filter(|b| !self.new_result.contains(b)).collect()
            },
            _ => Vec::new()
        }
    }

    fn prepare_compare(&mut self, new_repsonse: Vec<Vec<Binding>>, ts: usize) {
        if  self.ts < ts {
            mem::swap(&mut self.new_result, &mut self.old_result);
            self.new_result.clear();
            self.ts = ts;
        }
        new_repsonse.into_iter().for_each(|v| {
            self.new_result.insert(v);
            ()
        });
    }
}
#[cfg(test)]
mod tests{
    use crate::rsp::r2s::Relation2StreamOperator;
    use crate::rsp::r2s::StreamOperator::{DSTREAM, ISTREAM, RSTREAM};
    use crate::sparql::Binding;

    #[test]
    fn test_rstream(){
        let new_result = vec!(
            vec!(Binding{var:"?1".to_string(),val:"1".to_string()},
                                   Binding{var:"?2".to_string(),val:"2".to_string()}),
            vec!(Binding{var:"?1".to_string(),val:"1.2".to_string()},
                                   Binding{var:"?2".to_string(),val:"2.2".to_string()})
        );
        let mut s2r = Relation2StreamOperator::new(RSTREAM, 0);
        let expected_result = new_result.clone();

        assert_eq!(expected_result,s2r.eval(new_result,1));
    }
    #[test]
    fn test_dstream(){
        let old_result = vec!(
            vec!(Binding{var:"?1".to_string(),val:"1".to_string()},
                 Binding{var:"?2".to_string(),val:"2".to_string()}),
            vec!(Binding{var:"?1".to_string(),val:"1.2".to_string()},
                 Binding{var:"?2".to_string(),val:"2.2".to_string()})
        );
        let new_result = vec!(
            vec!(Binding{var:"?1".to_string(),val:"1".to_string()},
                 Binding{var:"?2".to_string(),val:"2".to_string()}),
            vec!(Binding{var:"?1".to_string(),val:"1.3".to_string()},
                 Binding{var:"?2".to_string(),val:"2.3".to_string()})
        );
        let expected_deletion = vec!(
            vec!(Binding{var:"?1".to_string(),val:"1.2".to_string()},
                 Binding{var:"?2".to_string(),val:"2.2".to_string()})
        );
        let mut s2r = Relation2StreamOperator::new(DSTREAM, 0);
        s2r.eval(old_result,1);

        assert_eq!(expected_deletion,s2r.eval(new_result,2));
    }
    #[test]
    fn test_istream(){
        let old_result = vec!(
            vec!(Binding{var:"?1".to_string(),val:"1".to_string()},
                 Binding{var:"?2".to_string(),val:"2".to_string()}),
            vec!(Binding{var:"?1".to_string(),val:"1.2".to_string()},
                 Binding{var:"?2".to_string(),val:"2.2".to_string()})
        );
        let new_result = vec!(
            vec!(Binding{var:"?1".to_string(),val:"1".to_string()},
                 Binding{var:"?2".to_string(),val:"2".to_string()}),
            vec!(Binding{var:"?1".to_string(),val:"1.3".to_string()},
                 Binding{var:"?2".to_string(),val:"2.3".to_string()})
        );
        let expected_deletion = vec!(
            vec!(Binding{var:"?1".to_string(),val:"1.3".to_string()},
                 Binding{var:"?2".to_string(),val:"2.3".to_string()})
        );
        let mut s2r = Relation2StreamOperator::new(ISTREAM, 0);
        s2r.eval(old_result,1);

        assert_eq!(expected_deletion,s2r.eval(new_result,2));
    }
}