var Encoder = /** @class */ (function () {
    function Encoder() {
        this.counter = 0;
        this.decoded = new Map();
        this.encoded = new Map();
    }
    Encoder.prototype.add = function (uri) {
        if (this.encoded.has(uri)) {
            var val = this.encoded.get(uri);
            if (val !== undefined) {
                return val;
            }
            else {
                return 0; //TODO fix
            }
        }
        else {
            this.encoded.set(uri, this.counter);
            this.decoded.set(this.counter, uri);
            this.counter += 1;
            return this.counter - 1;
        }
    };
    Encoder.prototype.decode = function (encoded) {
        return this.decoded.get(encoded);
    };
    return Encoder;
}());
var Var = /** @class */ (function () {
    function Var(encoded) {
        this.content = encoded;
    }
    Var.prototype.isTerm = function () {
        return false;
    };
    Var.prototype.isVar = function () {
        return true;
    };
    return Var;
}());
var Term = /** @class */ (function () {
    function Term(encoded) {
        this.content = encoded;
    }
    Term.prototype.isTerm = function () {
        return true;
    };
    Term.prototype.isVar = function () {
        return false;
    };
    return Term;
}());
var Triple = /** @class */ (function () {
    function Triple() {
    }
    Triple.from = function (s_string, p_string, o_string, encoder) {
        var new_triple = new Triple();
        new_triple.s = this.createVarOrTerm(s_string, encoder);
        new_triple.p = this.createVarOrTerm(p_string, encoder);
        new_triple.o = this.createVarOrTerm(o_string, encoder);
        return new_triple;
    };
    Triple.tripleFromEncoded = function (s_term, p_term, o_term) {
        var new_triple = new Triple();
        new_triple.s = s_term;
        new_triple.o = o_term;
        new_triple.p = p_term;
        return new_triple;
    };
    Triple.createVarOrTerm = function (item, encoder) {
        if (item.startsWith("?")) {
            return new Var(encoder.add(item));
        }
        else {
            return new Term(encoder.add(item));
        }
    };
    return Triple;
}());
var Rule = /** @class */ (function () {
    function Rule(head, body) {
        this.head = head;
        this.body = body;
    }
    return Rule;
}());
var TripleIndex = /** @class */ (function () {
    function TripleIndex() {
        this.triples = [];
        this.spo = new Map();
        this.pos = new Map();
        this.osp = new Map();
    }
    TripleIndex.prototype.len = function () {
        return this.triples.length;
    };
    TripleIndex.prototype.add = function (triple) {
        //spo
        if (!this.spo.has(triple.s.content)) {
            this.spo.set(triple.s.content, new Map());
        }
        if (!this.spo.get(triple.s.content) ? .has(triple.p.content) : ) {
            this.spo.get(triple.s.content) ? .set(triple.p.content, []) : ;
        }
        this.spo.get(triple.s.content) ? .get(triple.p.content) ? .push(triple.o.content) :  : ;
        //pos
        if (!this.pos.has(triple.p.content)) {
            this.pos.set(triple.p.content, new Map());
        }
        if (!this.pos.get(triple.p.content) ? .has(triple.o.content) : ) {
            this.pos.get(triple.p.content) ? .set(triple.o.content, []) : ;
        }
        this.spo.get(triple.p.content) ? .get(triple.o.content) ? .push(triple.s.content) :  : ;
        // osp
        if (!this.osp.has(triple.o.content)) {
            this.osp.set(triple.o.content, new Map());
        }
        if (!this.osp.get(triple.o.content) ? .has(triple.s.content) : ) {
            this.osp.get(triple.o.content) ? .set(triple.s.content, []) : ;
        }
        this.osp.get(triple.o.content) ? .get(triple.s.content) ? .push(triple.p.content) :  : ;
        this.triples.push(triple);
    };
    TripleIndex.prototype.contains = function (triple) {
        if (!this.osp.has(triple.o.content)) {
            return false;
        }
        else {
            if (!this.osp.get(triple.o.content) ? .has(triple.s.content) : ) {
                return false;
            }
            else {
                // @ts-ignore
                return this.osp.get(triple.o.content) ? .get(triple.s.content) ? .includes(triple.p.content) :  : ;
            }
        }
    };
    return TripleIndex;
}());
var RuleIndex = /** @class */ (function () {
    function RuleIndex() {
        this.s = new Map();
        this.o = new Map();
        this.p = new Map();
        this.sp = new Map();
        this.po = new Map();
        this.so = new Map();
        this.spo = [];
    }
    RuleIndex.prototype.len = function () {
        return this.spo.length;
    };
    RuleIndex.prototype.add = function (rule) {
        var _this = this;
        rule.body.forEach(function (triple) {
            var s = triple.s;
            var o = triple.o;
            var p = triple.p;
            //s match
            if (s.isTerm() && p.isVar() && o.isVar()) {
                if (!_this.s.has(s.content)) {
                    _this.s.set(s.content, []);
                }
                if (!_this.s.get(s.content) ? .includes(rule) : ) {
                    _this.s.get(s.content) ? .push(rule) : ;
                }
            }
            //p match
            if (s.isVar() && p.isTerm() && o.isVar()) {
                if (!_this.p.has(p.content)) {
                    _this.p.set(p.content, []);
                }
                if (!_this.p.get(p.content) ? .includes(rule) : ) {
                    _this.p.get(p.content) ? .push(rule) : ;
                }
            }
            //o match
            if (s.isVar() && p.isVar() && o.isTerm()) {
                if (!_this.o.has(s.content)) {
                    _this.o.set(s.content, []);
                }
                if (!_this.o.get(s.content) ? .includes(rule) : ) {
                    _this.o.get(s.content) ? .push(rule) : ;
                }
            }
            //sp match
            if (s.isTerm() && p.isTerm() && o.isVar()) {
                if (!_this.sp.has(s.content)) {
                    _this.sp.set(s.content, new Map());
                }
                if (!_this.sp.get(s.content) ? .has(p.content) : ) {
                    _this.sp.get(s.content) ? .set(p.content, []) : ;
                }
                if (!_this.sp.get(s.content) ? .get(p.content) ? .includes(rule) :  : ) {
                    _this.sp.get(s.content) ? .get(p.content) ? .push(rule) :  : ;
                }
            }
            //so match
            if (s.isTerm() && p.isVar() && o.isTerm()) {
                if (!_this.so.has(s.content)) {
                    _this.so.set(s.content, new Map());
                }
                if (!_this.so.get(s.content) ? .has(o.content) : ) {
                    _this.so.get(s.content) ? .set(o.content, []) : ;
                }
                if (!_this.so.get(s.content) ? .get(o.content) ? .includes(rule) :  : ) {
                    _this.so.get(s.content) ? .get(o.content) ? .push(rule) :  : ;
                }
            }
            //po match
            if (s.isVar() && p.isTerm() && o.isTerm()) {
                if (!_this.po.has(p.content)) {
                    _this.po.set(p.content, new Map());
                }
                if (!_this.po.get(p.content) ? .has(o.content) : ) {
                    _this.po.get(p.content) ? .set(o.content, []) : ;
                }
                if (!_this.po.get(p.content) ? .get(o.content) ? .includes(rule) :  : ) {
                    _this.po.get(p.content) ? .get(o.content) ? .push(rule) :  : ;
                }
            }
            //spo
            if (s.isVar() && p.isVar() && o.isVar()) {
                if (!_this.spo.includes(rule)) {
                    _this.spo.push(rule);
                }
            }
        });
    };
    RuleIndex.prototype.findMatch = function (triple) {
        var matchedTripels = [];
        //check s
        if (this.s.has(triple.s.content)) {
            this.s.get(triple.s.content) ? .forEach(function (t) { return matchedTripels.push(t); }) : ;
        }
        //check p
        if (this.p.has(triple.p.content)) {
            this.p.get(triple.p.content) ? .forEach(function (t) { return matchedTripels.push(t); }) : ;
        }
        //check o
        if (this.o.has(triple.o.content)) {
            this.o.get(triple.o.content) ? .forEach(function (t) { return matchedTripels.push(t); }) : ;
        }
        //check sp
        if (this.sp.has(triple.s.content)) {
            if (this.sp.get(triple.s.content) ? .has(triple.p.content) : ) {
                this.sp.get(triple.s.content) ? .get(triple.p.content) ? .forEach(function (t) { return matchedTripels.push(t); }) :  : ;
            }
        }
        //check so
        if (this.so.has(triple.s.content)) {
            if (this.so.get(triple.s.content) ? .has(triple.o.content) : ) {
                this.so.get(triple.s.content) ? .get(triple.o.content) ? .forEach(function (t) { return matchedTripels.push(t); }) :  : ;
            }
        }
        //check po
        if (this.po.has(triple.p.content)) {
            if (this.po.get(triple.p.content) ? .has(triple.o.content) : ) {
                this.po.get(triple.p.content) ? .get(triple.o.content) ? .forEach(function (t) { return matchedTripels.push(t); }) :  : ;
            }
        }
        this.spo.forEach(function (t) { return matchedTripels.push(t); });
        return matchedTripels;
    };
    return RuleIndex;
}());
var Binding = /** @class */ (function () {
    function Binding() {
        this.bindings = new Map();
    }
    Binding.prototype.add = function (var_name, term) {
        if (!this.bindings.has(var_name)) {
            this.bindings.set(var_name, []);
        }
        this.bindings.get(var_name) ? .push(term) : ;
    };
    Binding.prototype.len = function () {
        for (var _i = 0, _a = this.bindings.entries(); _i < _a.length; _i++) {
            var _b = _a[_i], key = _b[0], value = _b[1];
            return value.length;
        }
        return 0;
    };
    Binding.prototype.join = function (join_binding) {
        var left = join_binding;
        left = this;
        var right = join_binding;
        if (left.len() == 0) {
            return right;
        }
        if (right.len() == 0) {
            return left;
        }
        var result = new Binding();
        var left_bindings = left.bindings;
        var right_bindings = right.bindings;
        if (left.len() < right.len()) {
            right_bindings = this.bindings;
            left = join_binding;
        }
        var join_keys = [];
        for (var _i = 0, _a = left.bindings.keys(); _i < _a.length; _i++) {
            var key = _a[_i];
            if (right.bindings.has(key)) {
                join_keys.push(key);
            }
        }
        for (var left_c = 0; left_c < left.len(); left_c++) {
            for (var right_c = 0; right_c < right.len(); right_c++) {
                var match_keys = true;
                for (var _b = 0, join_keys_1 = join_keys; _b < join_keys_1.length; _b++) {
                    var join_key = join_keys_1[_b];
                    var left_term = left.bindings.get(join_key) ? .at(left_c) ? .content :  : ;
                    var right_term = right.bindings.get(join_key) ? .at(right_c) ? .content :  : ;
                    if (left_term != right_term) {
                        match_keys = false;
                        break;
                    }
                }
                if (match_keys) {
                    for (var _c = 0, _d = left.bindings.keys(); _c < _d.length; _c++) {
                        var key = _d[_c];
                        // @ts-ignore
                        result.add(key, left.bindings.get(key) ? .at(left_c) : );
                    }
                    for (var _e = 0, _f = right.bindings.keys(); _e < _f.length; _e++) {
                        var key = _f[_e];
                        if (!left.bindings.has(key)) {
                            // @ts-ignore
                            result.add(key, right.bindings.get(key) ? .at(right_c) : );
                        }
                    }
                }
            }
        }
        return result;
    };
    return Binding;
}());
var TripleStore = /** @class */ (function () {
    function TripleStore() {
        this.rules_index = new RuleIndex();
        this.rules = [];
        this.triple_index = new TripleIndex();
        this.encoder = new Encoder();
    }
    TripleStore.prototype.add = function (triple) {
        this.triple_index.add(triple);
    };
    TripleStore.prototype.add_rule = function (rule) {
        this.rules_index.add(rule);
        this.rules.push(rule);
    };
    TripleStore.prototype.query = function (query_triple, triple_counter) {
        var bindings = new Binding();
        var counter = triple_counter ?  ? this.triple_index.len() :  : ;
        console.log("checking ");
        for (var _i = 0, _a = this.triple_index.triples.slice(0, counter); _i < _a.length; _i++) {
            var triple = _a[_i];
            console.log(triple);
            if (query_triple.s.isVar()) {
                bindings.add(query_triple.s.content, triple.s);
            }
            else {
                if (query_triple.s.content != triple.s.content) {
                    break;
                }
            }
            if (query_triple.p.isVar()) {
                bindings.add(query_triple.p.content, triple.p);
            }
            else {
                if (query_triple.p.content != triple.p.content) {
                    break;
                }
            }
            if (query_triple.o.isVar()) {
                bindings.add(query_triple.o.content, triple.o);
            }
            else {
                if (query_triple.o.content != triple.o.content) {
                    break;
                }
            }
        }
        return bindings;
    };
    TripleStore.prototype.queryWithJoin = function (query_triples, triple_counter) {
        var bindings = new Binding();
        for (var _i = 0, query_triples_1 = query_triples; _i < query_triples_1.length; _i++) {
            var query_triple = query_triples_1[_i];
            var current_binding = this.query(query_triple, triple_counter);
            bindings = bindings.join(current_binding);
        }
        return bindings;
    };
    TripleStore.prototype.substituteRuleHead = function (head, binding) {
        var new_heads = [];
        var s, p, o;
        for (var result_counter = 0; result_counter < binding.len(); result_counter++) {
            if (head.s.isVar()) {
                s = binding.bindings.get(head.s.content) ? .at(result_counter) : ;
            }
            else {
                s = head.s;
            }
            if (head.p.isVar()) {
                p = binding.bindings.get(head.p.content) ? .at(result_counter) : ;
            }
            else {
                p = head.p;
            }
            if (head.o.isVar()) {
                o = binding.bindings.get(head.o.content) ? .at(result_counter) : ;
            }
            else {
                o = head.o;
            }
            // @ts-ignore
            new_heads.push(Triple.tripleFromEncoded(s, p, o));
        }
        return new_heads;
    };
    TripleStore.prototype.materialize = function () {
        var inferred = new Array();
        var counter = 0;
        while (counter < this.triple_index.triples.length) {
            var process_triple = this.triple_index.triples.at(counter);
            if (process_triple !== undefined) {
                var matching_rules = this.rules_index.findMatch(process_triple);
                var new_triples = [];
                for (var _i = 0, matching_rules_1 = matching_rules; _i < matching_rules_1.length; _i++) {
                    var rule = matching_rules_1[_i];
                    var temp_bindings = this.queryWithJoin(rule.body, counter + 1);
                    var new_heads = this.substituteRuleHead(rule.head, temp_bindings);
                    for (var _a = 0, new_heads_1 = new_heads; _a < new_heads_1.length; _a++) {
                        var new_head = new_heads_1[_a];
                        new_triples.push(new_head);
                    }
                }
                for (var _b = 0, new_triples_1 = new_triples; _b < new_triples_1.length; _b++) {
                    var new_triple = new_triples_1[_b];
                    if (!this.triple_index.contains(new_triple)) {
                        this.triple_index.add(new_triple);
                        inferred.push(new_triple);
                    }
                }
            }
            counter += 1;
        }
        return inferred;
    };
    return TripleStore;
}());
var startTime = performance.now();
var encoder = new Encoder();
var triple_store = new TripleStore();
triple_store.add(Triple.from("s1", "p", "o0", encoder));
for (var i = 0; i < 100000; i++) {
    var triple_head = Triple.from("?s1", "p", "o" + (i + 1), encoder);
    var triple_body1 = Triple.from("?s1", "p", "o" + i, encoder);
    var rule = new Rule(triple_head, [triple_body1]);
    triple_store.add_rule(rule);
}
var inferred = triple_store.materialize();
console.log("inferred");
console.log(inferred.length);
var endTime = performance.now();
console.log("Call to doSomething took " + (endTime - startTime) + " milliseconds");
