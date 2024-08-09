
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::{Rule, Triple, TripleStore, TermImpl, VarOrTerm, RuleIndex, TripleIndex, Encoder, SimpleQueryEngine, QueryEngine, Parser, BackwardChainer};
    use crate::csprite::CSprite;
    use crate::reasoner::Reasoner;

    #[test]
    fn test_eval_backward_rule(){
        let data="@prefix test: <http://test.org/> \n\
        <http://example2.com/mlModel1> a test:MLModel.\n\
                <http://example2.com/mlModel1> a test:Action.\n\
                <http://example2.com/mlModel1> test:hasInput <http://example2.com/relational_normalied>.\n\
                <http://example2.com/mlModel1> test:requirement <http://example2.com/normalisation>.\n\
                <http://example2.com/normalizer> a test:MLModel.\n\
                <http://example2.com/normalizer> test:hasInput <http://example2.com/relational_nonnormalized>.\n\
                <http://example2.com/normalizer> test:hasOutput <http://example2.com/relational_normalied>.\n\
                <http://example2.com/normalizer> test:hasFunction <http://example2.com/normalisation>.\n\
                <http://example2.com/temperatureSensor> a test:Source.\n\
                <http://example2.com/temperatureSensor> test:hasOutput <http://example2.com/relational_nonnormalized>.\n\
                {?source a test:Source. ?source test:hasOutput ?output. }=>{?source a test:Test. }.\n\
             {?x test:hasInput ?input. ?x test:requirement ?req. ?y test:hasOutput ?input. ?y test:hasFunction ?req.}=>{?y test:hasInput ?input.}\n\
             {?source a test:Source. ?source test:hasOutput ?output. ?y test:hasInput ?output }=>{?source a test:NeededInput. }.";

        let mut store = TripleStore::from(data);
        let backward_head = Triple{s:VarOrTerm::new_var("?newVar".to_string()),p:VarOrTerm::new_term("a".to_string()),o:VarOrTerm::new_term("test:Test".to_string()), g: None};
        let var_encoded= Encoder::add("?newVar".to_string());
        let result_encoded = Encoder::add("<http://example2.com/temperatureSensor>".to_string());

        let  bindings = BackwardChainer::eval_backward(&store.triple_index, &store.rules_index, &backward_head);
        println!("Bindings {}", TripleStore::decode_bindings(&bindings));
        let result_bindings = HashMap::from([
            (var_encoded, Vec::from([result_encoded]))
        ]);
        assert_eq!(result_bindings.get(&12), bindings.get(&12));
    }
    #[test]
    fn test_eval_forward_rule(){
        let data="@prefix test: <http://test.org/> \n\
        <http://example2.com/mlModel1> a test:MLModel.\n\
                <http://example2.com/mlModel1> a test:Action.\n\
                <http://example2.com/mlModel1> test:hasInput <http://example2.com/relational_normalied>.\n\
                <http://example2.com/mlModel1> test:requirement <http://example2.com/normalisation>.\n\
                <http://example2.com/normalizer> a test:MLModel.\n\
                <http://example2.com/normalizer> test:hasInput <http://example2.com/relational_nonnormalized>.\n\
                <http://example2.com/normalizer> test:hasOutput <http://example2.com/relational_normalied>.\n\
                <http://example2.com/normalizer> test:hasFunction <http://example2.com/normalisation>.\n\
                <http://example2.com/temperatureSensor> a test:Source.\n\
                <http://example2.com/temperatureSensor> test:hasOutput <http://example2.com/relational_nonnormalized>.\n\
                {?source a test:Source. ?source test:hasOutput ?output. }=>{?source a test:Test. }.\n\
             {?x test:hasInput ?input. ?x test:requirement ?req. ?y test:hasOutput ?input. ?y test:hasFunction ?req.}=>{?y test:hasInput ?input.}\n\
             {?source a test:Source. ?source test:hasOutput ?output. ?y test:hasInput ?output }=>{?source <http://test.org/defines> <http://test.org/NeededInput>. }.";
        let mut store = TripleStore::from(data);
        println!("store size {:?}", store.len());
        store.materialize();
        println!("store size {:?}", store.len());
        match store.query("Select * WHERE{?s <http://test.org/defines> ?o}"){
            Ok(result)=>assert_ne!(0, result.len()),
            Err(err)=>assert_eq!(0, 1)
        }

    }
}