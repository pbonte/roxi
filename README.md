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

## Using RoXi with Javascript/Typescript from NPM in your Node project

You can add find the Javascript bindings directly on [NPM](https://www.npmjs.com/package/roxi-js?activeTab=readme)
`npm i roxi-js`

See the examples below on how to use RoXi in JS mode

## Building RoXi for Javascript/Typescript usage

Make sure that you have `wasm-pack`, `cargo-generate` and `npm` installed. Instructions to install those can be found [here](https://rustwasm.github.io/book/game-of-life/setup.html).

You can use roxi both inside a browser as well as a Node JS module.

### Using Roxi inside a browser.

```
cd js
wasm-pack build
```

A `pkg` folder will be generated which contains the generated web assembly modules which can be used in the browser. You can install the package inside your application with `npm install --save-dev /path/to/roxi/js/pkg` More information and a tutorial regarding using Webassembly within webpages using webpack can be found [here](https://rustwasm.github.io/book/game-of-life/hello-world.html#putting-it-into-a-web-page)

### Using Roxi as a Node Module.

```
cd js
wasm-pack build --target nodejs
```
You can install the package with `npm install --save-dev path/to/roxi/js/pkg`. The package will be found in your dependencies as,
```
  "dependencies": {
    "roxi-js": "file:../roxi/js/pkg"
  }
```

Go to package.json of your project and add:

```
  "type": "module"
```

## Examples of RoXi in JS/TS mode

Example usage when using the static reasoner:

```javascript
import {RoxiReasoner} from "roxi-js";
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
Example usage when using the RSP engine:

```javascript
import {JSRSPEngine} from "roxi-js";
// callback function
function callback(val) {
    console.log(val);
}
let width = 10;
let slide = 2;
let rules = "@prefix test: <http://test/>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n {?x test:isIn ?y. ?y test:isIn ?z. }=>{?x test:isIn ?z.}";
let abox = "";
let query = "Select * WHERE{ ?x <http://test/isIn> ?y}";
// create the engine
let rspEngine = JSRSPEngine.new(width,slide,rules,abox,query,callback);
// add some data
let event = "<http://test/0> <http://test/isIn> <http://test/1>.";
let currentTimeStamp = 0;
rspEngine.add(event, currentTimeStamp);
...
let event = ... ;
currentTimeStamp += 1;
rspEngine.add(event, currentTimeStamp);

```




