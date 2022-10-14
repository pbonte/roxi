(window["webpackJsonp"] = window["webpackJsonp"] || []).push([[0],{

/***/ "../pkg/js.js":
/*!********************!*\
  !*** ../pkg/js.js ***!
  \********************/
/*! exports provided: RoxiReasoner, __wbindgen_throw */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./js_bg.wasm */ \"../pkg/js_bg.wasm\");\n/* harmony import */ var _js_bg_js__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./js_bg.js */ \"../pkg/js_bg.js\");\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"RoxiReasoner\", function() { return _js_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"RoxiReasoner\"]; });\n\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_throw\", function() { return _js_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"__wbindgen_throw\"]; });\n\n\n\n\n//# sourceURL=webpack:///../pkg/js.js?");

/***/ }),

/***/ "../pkg/js_bg.js":
/*!***********************!*\
  !*** ../pkg/js_bg.js ***!
  \***********************/
/*! exports provided: RoxiReasoner, __wbindgen_throw */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* WEBPACK VAR INJECTION */(function(module) {/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"RoxiReasoner\", function() { return RoxiReasoner; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_throw\", function() { return __wbindgen_throw; });\n/* harmony import */ var _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./js_bg.wasm */ \"../pkg/js_bg.wasm\");\n\n\nconst lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;\n\nlet cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });\n\ncachedTextDecoder.decode();\n\nlet cachedUint8Memory0;\nfunction getUint8Memory0() {\n    if (cachedUint8Memory0.byteLength === 0) {\n        cachedUint8Memory0 = new Uint8Array(_js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer);\n    }\n    return cachedUint8Memory0;\n}\n\nfunction getStringFromWasm0(ptr, len) {\n    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));\n}\n\nlet WASM_VECTOR_LEN = 0;\n\nconst lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;\n\nlet cachedTextEncoder = new lTextEncoder('utf-8');\n\nconst encodeString = (typeof cachedTextEncoder.encodeInto === 'function'\n    ? function (arg, view) {\n    return cachedTextEncoder.encodeInto(arg, view);\n}\n    : function (arg, view) {\n    const buf = cachedTextEncoder.encode(arg);\n    view.set(buf);\n    return {\n        read: arg.length,\n        written: buf.length\n    };\n});\n\nfunction passStringToWasm0(arg, malloc, realloc) {\n\n    if (realloc === undefined) {\n        const buf = cachedTextEncoder.encode(arg);\n        const ptr = malloc(buf.length);\n        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);\n        WASM_VECTOR_LEN = buf.length;\n        return ptr;\n    }\n\n    let len = arg.length;\n    let ptr = malloc(len);\n\n    const mem = getUint8Memory0();\n\n    let offset = 0;\n\n    for (; offset < len; offset++) {\n        const code = arg.charCodeAt(offset);\n        if (code > 0x7F) break;\n        mem[ptr + offset] = code;\n    }\n\n    if (offset !== len) {\n        if (offset !== 0) {\n            arg = arg.slice(offset);\n        }\n        ptr = realloc(ptr, len, len = offset + arg.length * 3);\n        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);\n        const ret = encodeString(arg, view);\n\n        offset += ret.written;\n    }\n\n    WASM_VECTOR_LEN = offset;\n    return ptr;\n}\n\nlet cachedInt32Memory0;\nfunction getInt32Memory0() {\n    if (cachedInt32Memory0.byteLength === 0) {\n        cachedInt32Memory0 = new Int32Array(_js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer);\n    }\n    return cachedInt32Memory0;\n}\n/**\n*/\nclass RoxiReasoner {\n\n    static __wrap(ptr) {\n        const obj = Object.create(RoxiReasoner.prototype);\n        obj.ptr = ptr;\n\n        return obj;\n    }\n\n    __destroy_into_raw() {\n        const ptr = this.ptr;\n        this.ptr = 0;\n\n        return ptr;\n    }\n\n    free() {\n        const ptr = this.__destroy_into_raw();\n        _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbg_roxireasoner_free\"](ptr);\n    }\n    /**\n    * @returns {RoxiReasoner}\n    */\n    static new() {\n        const ret = _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"roxireasoner_new\"]();\n        return RoxiReasoner.__wrap(ret);\n    }\n    /**\n    * @param {string} abox\n    */\n    add_abox(abox) {\n        const ptr0 = passStringToWasm0(abox, _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_malloc\"], _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_realloc\"]);\n        const len0 = WASM_VECTOR_LEN;\n        _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"roxireasoner_add_abox\"](this.ptr, ptr0, len0);\n    }\n    /**\n    * @param {string} rules\n    */\n    add_rules(rules) {\n        const ptr0 = passStringToWasm0(rules, _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_malloc\"], _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_realloc\"]);\n        const len0 = WASM_VECTOR_LEN;\n        _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"roxireasoner_add_rules\"](this.ptr, ptr0, len0);\n    }\n    /**\n    * @returns {number}\n    */\n    len_abox() {\n        const ret = _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"roxireasoner_len_abox\"](this.ptr);\n        return ret >>> 0;\n    }\n    /**\n    */\n    materialize() {\n        _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"roxireasoner_materialize\"](this.ptr);\n    }\n    /**\n    * @returns {string}\n    */\n    get_abox_dump() {\n        try {\n            const retptr = _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_add_to_stack_pointer\"](-16);\n            _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"roxireasoner_get_abox_dump\"](retptr, this.ptr);\n            var r0 = getInt32Memory0()[retptr / 4 + 0];\n            var r1 = getInt32Memory0()[retptr / 4 + 1];\n            return getStringFromWasm0(r0, r1);\n        } finally {\n            _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_add_to_stack_pointer\"](16);\n            _js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_free\"](r0, r1);\n        }\n    }\n}\n\nfunction __wbindgen_throw(arg0, arg1) {\n    throw new Error(getStringFromWasm0(arg0, arg1));\n};\n\ncachedInt32Memory0 = new Int32Array(_js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer);\ncachedUint8Memory0 = new Uint8Array(_js_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer);\n\n\n/* WEBPACK VAR INJECTION */}.call(this, __webpack_require__(/*! ./../web/node_modules/webpack/buildin/harmony-module.js */ \"./node_modules/webpack/buildin/harmony-module.js\")(module)))\n\n//# sourceURL=webpack:///../pkg/js_bg.js?");

/***/ }),

/***/ "../pkg/js_bg.wasm":
/*!*************************!*\
  !*** ../pkg/js_bg.wasm ***!
  \*************************/
/*! exports provided: memory, __wbg_roxireasoner_free, roxireasoner_new, roxireasoner_add_abox, roxireasoner_add_rules, roxireasoner_len_abox, roxireasoner_materialize, roxireasoner_get_abox_dump, __wbindgen_malloc, __wbindgen_realloc, __wbindgen_add_to_stack_pointer, __wbindgen_free */
/***/ (function(module, exports, __webpack_require__) {

eval("\"use strict\";\n// Instantiate WebAssembly module\nvar wasmExports = __webpack_require__.w[module.i];\n__webpack_require__.r(exports);\n// export exports from WebAssembly module\nfor(var name in wasmExports) if(name != \"__webpack_init__\") exports[name] = wasmExports[name];\n// exec imports from WebAssembly module (for esm order)\n/* harmony import */ var m0 = __webpack_require__(/*! ./js_bg.js */ \"../pkg/js_bg.js\");\n\n\n// exec wasm module\nwasmExports[\"__webpack_init__\"]()\n\n//# sourceURL=webpack:///../pkg/js_bg.wasm?");

/***/ }),

/***/ "./index.js":
/*!******************!*\
  !*** ./index.js ***!
  \******************/
/*! no exports provided */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var roxi__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! roxi */ \"../pkg/js.js\");\n\n\n\nlet abox= \"<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .\";\nlet tbox = \"@prefix test: <http://www.test.be/test#>.\\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\\n {?s rdf:type test:SubClass. }=>{?s rdf:type test:SuperType.}\"\ndocument.getElementById('aboxContent').value = abox;\ndocument.getElementById('rulesContent').value = tbox;\n\n\n\nconst reasoningButton = document.getElementById(\"startReasoning\");\n\nconst startReasoning = () => {\n\n    const reasoner = roxi__WEBPACK_IMPORTED_MODULE_0__[\"RoxiReasoner\"].new();\n\n    let abox_new= document.getElementById('aboxContent').value;\n    let tbox_new = document.getElementById('rulesContent').value;\n    let startTime = new Date();\n\n    reasoner.add_abox(abox_new);\n    reasoner.add_rules(tbox_new);\n    reasoner.materialize();\n    let endTime = new Date();\n    let difftime = endTime-startTime ;\n    document.getElementById('results').value = reasoner.get_abox_dump();\n    document.getElementById('timeResults').innerHTML = difftime + \" ms\";\n};\n\n\n\nreasoningButton.addEventListener(\"click\", event => {\n    startReasoning();\n});\n\n//# sourceURL=webpack:///./index.js?");

/***/ }),

/***/ "./node_modules/webpack/buildin/harmony-module.js":
/*!*******************************************!*\
  !*** (webpack)/buildin/harmony-module.js ***!
  \*******************************************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("module.exports = function(originalModule) {\n\tif (!originalModule.webpackPolyfill) {\n\t\tvar module = Object.create(originalModule);\n\t\t// module.parent = undefined by default\n\t\tif (!module.children) module.children = [];\n\t\tObject.defineProperty(module, \"loaded\", {\n\t\t\tenumerable: true,\n\t\t\tget: function() {\n\t\t\t\treturn module.l;\n\t\t\t}\n\t\t});\n\t\tObject.defineProperty(module, \"id\", {\n\t\t\tenumerable: true,\n\t\t\tget: function() {\n\t\t\t\treturn module.i;\n\t\t\t}\n\t\t});\n\t\tObject.defineProperty(module, \"exports\", {\n\t\t\tenumerable: true\n\t\t});\n\t\tmodule.webpackPolyfill = 1;\n\t}\n\treturn module;\n};\n\n\n//# sourceURL=webpack:///(webpack)/buildin/harmony-module.js?");

/***/ })

}]);