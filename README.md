# RoXi

Reactive Reasoning on top of [oxigraph](https://github.com/oxigraph/oxigraph)


RoXi can be included as a library, run in server mode through CLI or in the browser using web assembly.

## RoXi Server

How to build Roxi in server mode:
```
cd server
cargo build --release
./target/release/server --abox <ABOX> --tbox <TBOX> --query <QUERY>
```
The following parameters can be defined:
1. `--abox` file location to abox statements. File in TTL format (.ttl) supported.
2. `--tbox` file location to tbox statements. Files in TTL format (.ttl) and N3 Logic (.n3) supported.
3. `--query` string representing SPARQL query

For example:
```
./target/release/server --abox examples/abox.ttl --tbox examples/rules.n3 --query "Select * WHERE {?S ?P ?O}"
```
