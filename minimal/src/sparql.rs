use spargebra::Query;
use spargebra::Query::Select;

#[test]
fn test_sparql_parser(){

    let query_str = "CONSTRUCT {?s ?p ?o} WHERE { GRAPH ?w { ?s ?p ?o . } }";
    let query = Query::parse(query_str, None).unwrap();
    println!("{}",query.to_sse());

    match query {
        spargebra::Query::Select {
            pattern, base_iri, ..
        } => {
            println!("Select query");
        }
        spargebra::Query::Ask {
            pattern, base_iri, ..
        } => {
            println!("Ask query");
        }
        spargebra::Query::Construct {
            template,
            pattern,
            base_iri,
            ..
        } => {
            println!("Construct query");
        }
        spargebra::Query::Describe {
            pattern, base_iri, ..
        } => {
            println!("Describe query");
        }
    }
}