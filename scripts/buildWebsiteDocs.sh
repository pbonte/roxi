#! /bin/bash
cd ../js/
wasm-pack build
cd web
npm run build
cd dist
cp * ../../../docs
