# RoXi

RoXi provides a uniform framework for Reactive Reasoning applications including:
- Incremental maintenance
- RDF Stream Processing
- Temporal Reasoning (TODO)

RoXi uses some of the internals of [oxigraph](https://github.com/oxigraph/oxigraph), including [sparqlalgebra](https://crates.io/crates/spargebra) and [OxRDF](https://crates.io/crates/oxrdf).


RoXi can be included as a library, run in server mode through CLI or in the browser using web assembly.
You can try it out in your [own browser](https://pbonte.github.io/roxi/index.html)!

## RoXi Server

How to build RoXi in server mode:
```
cd server
cargo build --release
cd ..
./target/release/server --abox <ABOX> --tbox <TBOX> 
```
The following parameters can be defined:
1. `--abox` file location to abox statements. File in TTL format (.ttl) supported.
2. `--tbox` file location to tbox statements. Files in TTL format (.ttl) and N3 Logic (.n3) supported.
3`--trace` [optional] boolean for printing reasoning traces 

For example:
```
./target/release/server --abox examples/abox.ttl --tbox examples/rules.n3 
```

## Roxi JS lib
Make sure to have `wasm-pack`, `cargo-generate` and `npm` installed. Instruction can be found [here](https://rustwasm.github.io/book/game-of-life/setup.html).
How to build RoXi through web assembly:
```
cd js
wasm-pack build
```
This will generate a `pkg` folder. Now you can add RoXi as a dependency in your npm project:
```
{
  // ...
  "dependencies": {
   "roxi": "file:../pkg",
    // ...
  }
}
```
Example usage:
```
import {RoxiReasoner} from "roxi";
// create the reasoner
const reasoner = RoxiReasoner.new();
// add ABox 
reasoner.add_abox("<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .");
// Add rules
reasoner.add_rules("@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n {?s rdf:type test:SubClass. }=>{?s rdf:type test:SuperType.}");
// perform materialization through forward chaining
reasoner.materialize();
// log a dump of the materialized abox
console.log(reasoner.get_abox_dump());
```

