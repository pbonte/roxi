"use strict";
var __values = (this && this.__values) || function(o) {
    var s = typeof Symbol === "function" && Symbol.iterator, m = s && o[s], i = 0;
    if (m) return m.call(o);
    if (o && typeof o.length === "number") return {
        next: function () {
            if (o && i >= o.length) o = void 0;
            return { value: o && o[i++], done: !o };
        }
    };
    throw new TypeError(s ? "Object is not iterable." : "Symbol.iterator is not defined.");
};
var __read = (this && this.__read) || function (o, n) {
    var m = typeof Symbol === "function" && o[Symbol.iterator];
    if (!m) return o;
    var i = m.call(o), r, ar = [], e;
    try {
        while ((n === void 0 || n-- > 0) && !(r = i.next()).done) ar.push(r.value);
    }
    catch (error) { e = { error: error }; }
    finally {
        try {
            if (r && !r.done && (m = i["return"])) m.call(i);
        }
        finally { if (e) throw e.error; }
    }
    return ar;
};
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
        var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k, _l, _m;
        //spo
        if (!this.spo.has(triple.s.content)) {
            this.spo.set(triple.s.content, new Map());
        }
        if (!((_a = this.spo.get(triple.s.content)) === null || _a === void 0 ? void 0 : _a.has(triple.p.content))) {
            (_b = this.spo.get(triple.s.content)) === null || _b === void 0 ? void 0 : _b.set(triple.p.content, []);
        }
        (_d = (_c = this.spo.get(triple.s.content)) === null || _c === void 0 ? void 0 : _c.get(triple.p.content)) === null || _d === void 0 ? void 0 : _d.push(triple.o.content);
        //pos
        if (!this.pos.has(triple.p.content)) {
            this.pos.set(triple.p.content, new Map());
        }
        if (!((_e = this.pos.get(triple.p.content)) === null || _e === void 0 ? void 0 : _e.has(triple.o.content))) {
            (_f = this.pos.get(triple.p.content)) === null || _f === void 0 ? void 0 : _f.set(triple.o.content, []);
        }
        (_h = (_g = this.spo.get(triple.p.content)) === null || _g === void 0 ? void 0 : _g.get(triple.o.content)) === null || _h === void 0 ? void 0 : _h.push(triple.s.content);
        // osp
        if (!this.osp.has(triple.o.content)) {
            this.osp.set(triple.o.content, new Map());
        }
        if (!((_j = this.osp.get(triple.o.content)) === null || _j === void 0 ? void 0 : _j.has(triple.s.content))) {
            (_k = this.osp.get(triple.o.content)) === null || _k === void 0 ? void 0 : _k.set(triple.s.content, []);
        }
        (_m = (_l = this.osp.get(triple.o.content)) === null || _l === void 0 ? void 0 : _l.get(triple.s.content)) === null || _m === void 0 ? void 0 : _m.push(triple.p.content);
        this.triples.push(triple);
    };
    TripleIndex.prototype.contains = function (triple) {
        var _a, _b, _c;
        if (!this.osp.has(triple.o.content)) {
            return false;
        }
        else {
            if (!((_a = this.osp.get(triple.o.content)) === null || _a === void 0 ? void 0 : _a.has(triple.s.content))) {
                return false;
            }
            else {
                // @ts-ignore
                return (_c = (_b = this.osp.get(triple.o.content)) === null || _b === void 0 ? void 0 : _b.get(triple.s.content)) === null || _c === void 0 ? void 0 : _c.includes(triple.p.content);
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
            var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k, _l, _m, _o, _p, _q, _r, _s, _t, _u, _v, _w, _x, _y, _z;
            var s = triple.s;
            var o = triple.o;
            var p = triple.p;
            //s match
            if (s.isTerm() && p.isVar() && o.isVar()) {
                if (!_this.s.has(s.content)) {
                    _this.s.set(s.content, []);
                }
                if (!((_a = _this.s.get(s.content)) === null || _a === void 0 ? void 0 : _a.includes(rule))) {
                    (_b = _this.s.get(s.content)) === null || _b === void 0 ? void 0 : _b.push(rule);
                }
            }
            //p match
            if (s.isVar() && p.isTerm() && o.isVar()) {
                if (!_this.p.has(p.content)) {
                    _this.p.set(p.content, []);
                }
                if (!((_c = _this.p.get(p.content)) === null || _c === void 0 ? void 0 : _c.includes(rule))) {
                    (_d = _this.p.get(p.content)) === null || _d === void 0 ? void 0 : _d.push(rule);
                }
            }
            //o match
            if (s.isVar() && p.isVar() && o.isTerm()) {
                if (!_this.o.has(s.content)) {
                    _this.o.set(s.content, []);
                }
                if (!((_e = _this.o.get(s.content)) === null || _e === void 0 ? void 0 : _e.includes(rule))) {
                    (_f = _this.o.get(s.content)) === null || _f === void 0 ? void 0 : _f.push(rule);
                }
            }
            //sp match
            if (s.isTerm() && p.isTerm() && o.isVar()) {
                if (!_this.sp.has(s.content)) {
                    _this.sp.set(s.content, new Map());
                }
                if (!((_g = _this.sp.get(s.content)) === null || _g === void 0 ? void 0 : _g.has(p.content))) {
                    (_h = _this.sp.get(s.content)) === null || _h === void 0 ? void 0 : _h.set(p.content, []);
                }
                if (!((_k = (_j = _this.sp.get(s.content)) === null || _j === void 0 ? void 0 : _j.get(p.content)) === null || _k === void 0 ? void 0 : _k.includes(rule))) {
                    (_m = (_l = _this.sp.get(s.content)) === null || _l === void 0 ? void 0 : _l.get(p.content)) === null || _m === void 0 ? void 0 : _m.push(rule);
                }
            }
            //so match
            if (s.isTerm() && p.isVar() && o.isTerm()) {
                if (!_this.so.has(s.content)) {
                    _this.so.set(s.content, new Map());
                }
                if (!((_o = _this.so.get(s.content)) === null || _o === void 0 ? void 0 : _o.has(o.content))) {
                    (_p = _this.so.get(s.content)) === null || _p === void 0 ? void 0 : _p.set(o.content, []);
                }
                if (!((_r = (_q = _this.so.get(s.content)) === null || _q === void 0 ? void 0 : _q.get(o.content)) === null || _r === void 0 ? void 0 : _r.includes(rule))) {
                    (_t = (_s = _this.so.get(s.content)) === null || _s === void 0 ? void 0 : _s.get(o.content)) === null || _t === void 0 ? void 0 : _t.push(rule);
                }
            }
            //po match
            if (s.isVar() && p.isTerm() && o.isTerm()) {
                if (!_this.po.has(p.content)) {
                    _this.po.set(p.content, new Map());
                }
                if (!((_u = _this.po.get(p.content)) === null || _u === void 0 ? void 0 : _u.has(o.content))) {
                    (_v = _this.po.get(p.content)) === null || _v === void 0 ? void 0 : _v.set(o.content, []);
                }
                if (!((_x = (_w = _this.po.get(p.content)) === null || _w === void 0 ? void 0 : _w.get(o.content)) === null || _x === void 0 ? void 0 : _x.includes(rule))) {
                    (_z = (_y = _this.po.get(p.content)) === null || _y === void 0 ? void 0 : _y.get(o.content)) === null || _z === void 0 ? void 0 : _z.push(rule);
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
        var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k, _l, _m;
        var matchedTripels = [];
        //check s
        if (this.s.has(triple.s.content)) {
            (_a = this.s.get(triple.s.content)) === null || _a === void 0 ? void 0 : _a.forEach(function (t) { return matchedTripels.push(t); });
        }
        //check p
        if (this.p.has(triple.p.content)) {
            (_b = this.p.get(triple.p.content)) === null || _b === void 0 ? void 0 : _b.forEach(function (t) { return matchedTripels.push(t); });
        }
        //check o
        if (this.o.has(triple.o.content)) {
            (_c = this.o.get(triple.o.content)) === null || _c === void 0 ? void 0 : _c.forEach(function (t) { return matchedTripels.push(t); });
        }
        //check sp
        if (this.sp.has(triple.s.content)) {
            if ((_d = this.sp.get(triple.s.content)) === null || _d === void 0 ? void 0 : _d.has(triple.p.content)) {
                (_f = (_e = this.sp.get(triple.s.content)) === null || _e === void 0 ? void 0 : _e.get(triple.p.content)) === null || _f === void 0 ? void 0 : _f.forEach(function (t) { return matchedTripels.push(t); });
            }
        }
        //check so
        if (this.so.has(triple.s.content)) {
            if ((_g = this.so.get(triple.s.content)) === null || _g === void 0 ? void 0 : _g.has(triple.o.content)) {
                (_j = (_h = this.so.get(triple.s.content)) === null || _h === void 0 ? void 0 : _h.get(triple.o.content)) === null || _j === void 0 ? void 0 : _j.forEach(function (t) { return matchedTripels.push(t); });
            }
        }
        //check po
        if (this.po.has(triple.p.content)) {
            if ((_k = this.po.get(triple.p.content)) === null || _k === void 0 ? void 0 : _k.has(triple.o.content)) {
                (_m = (_l = this.po.get(triple.p.content)) === null || _l === void 0 ? void 0 : _l.get(triple.o.content)) === null || _m === void 0 ? void 0 : _m.forEach(function (t) { return matchedTripels.push(t); });
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
        var _a;
        if (!this.bindings.has(var_name)) {
            this.bindings.set(var_name, []);
        }
        (_a = this.bindings.get(var_name)) === null || _a === void 0 ? void 0 : _a.push(term);
    };
    Binding.prototype.len = function () {
        var e_1, _a;
        try {
            for (var _b = __values(this.bindings.entries()), _c = _b.next(); !_c.done; _c = _b.next()) {
                var _d = __read(_c.value, 2), key = _d[0], value = _d[1];
                return value.length;
            }
        }
        catch (e_1_1) { e_1 = { error: e_1_1 }; }
        finally {
            try {
                if (_c && !_c.done && (_a = _b.return)) _a.call(_b);
            }
            finally { if (e_1) throw e_1.error; }
        }
        return 0;
    };
    Binding.prototype.join = function (join_binding) {
        var e_2, _a, e_3, _b, e_4, _c, e_5, _d;
        var _e, _f, _g, _h, _j, _k;
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
        try {
            for (var _l = __values(left.bindings.keys()), _m = _l.next(); !_m.done; _m = _l.next()) {
                var key = _m.value;
                if (right.bindings.has(key)) {
                    join_keys.push(key);
                }
            }
        }
        catch (e_2_1) { e_2 = { error: e_2_1 }; }
        finally {
            try {
                if (_m && !_m.done && (_a = _l.return)) _a.call(_l);
            }
            finally { if (e_2) throw e_2.error; }
        }
        for (var left_c = 0; left_c < left.len(); left_c++) {
            for (var right_c = 0; right_c < right.len(); right_c++) {
                var match_keys = true;
                try {
                    for (var join_keys_1 = (e_3 = void 0, __values(join_keys)), join_keys_1_1 = join_keys_1.next(); !join_keys_1_1.done; join_keys_1_1 = join_keys_1.next()) {
                        var join_key = join_keys_1_1.value;
                        var left_term = (_f = (_e = left.bindings.get(join_key)) === null || _e === void 0 ? void 0 : _e.at(left_c)) === null || _f === void 0 ? void 0 : _f.content;
                        var right_term = (_h = (_g = right.bindings.get(join_key)) === null || _g === void 0 ? void 0 : _g.at(right_c)) === null || _h === void 0 ? void 0 : _h.content;
                        if (left_term != right_term) {
                            match_keys = false;
                            break;
                        }
                    }
                }
                catch (e_3_1) { e_3 = { error: e_3_1 }; }
                finally {
                    try {
                        if (join_keys_1_1 && !join_keys_1_1.done && (_b = join_keys_1.return)) _b.call(join_keys_1);
                    }
                    finally { if (e_3) throw e_3.error; }
                }
                if (match_keys) {
                    try {
                        for (var _o = (e_4 = void 0, __values(left.bindings.keys())), _p = _o.next(); !_p.done; _p = _o.next()) {
                            var key = _p.value;
                            // @ts-ignore
                            result.add(key, (_j = left.bindings.get(key)) === null || _j === void 0 ? void 0 : _j.at(left_c));
                        }
                    }
                    catch (e_4_1) { e_4 = { error: e_4_1 }; }
                    finally {
                        try {
                            if (_p && !_p.done && (_c = _o.return)) _c.call(_o);
                        }
                        finally { if (e_4) throw e_4.error; }
                    }
                    try {
                        for (var _q = (e_5 = void 0, __values(right.bindings.keys())), _r = _q.next(); !_r.done; _r = _q.next()) {
                            var key = _r.value;
                            if (!left.bindings.has(key)) {
                                // @ts-ignore
                                result.add(key, (_k = right.bindings.get(key)) === null || _k === void 0 ? void 0 : _k.at(right_c));
                            }
                        }
                    }
                    catch (e_5_1) { e_5 = { error: e_5_1 }; }
                    finally {
                        try {
                            if (_r && !_r.done && (_d = _q.return)) _d.call(_q);
                        }
                        finally { if (e_5) throw e_5.error; }
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
        var e_6, _a;
        var bindings = new Binding();
        var counter = triple_counter !== null && triple_counter !== void 0 ? triple_counter : this.triple_index.len();
        try {
            for (var _b = __values(this.triple_index.triples.slice(0, counter)), _c = _b.next(); !_c.done; _c = _b.next()) {
                var triple = _c.value;
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
        }
        catch (e_6_1) { e_6 = { error: e_6_1 }; }
        finally {
            try {
                if (_c && !_c.done && (_a = _b.return)) _a.call(_b);
            }
            finally { if (e_6) throw e_6.error; }
        }
        return bindings;
    };
    TripleStore.prototype.queryWithJoin = function (query_triples, triple_counter) {
        var e_7, _a;
        var bindings = new Binding();
        try {
            for (var query_triples_1 = __values(query_triples), query_triples_1_1 = query_triples_1.next(); !query_triples_1_1.done; query_triples_1_1 = query_triples_1.next()) {
                var query_triple = query_triples_1_1.value;
                var current_binding = this.query(query_triple, triple_counter);
                bindings = bindings.join(current_binding);
            }
        }
        catch (e_7_1) { e_7 = { error: e_7_1 }; }
        finally {
            try {
                if (query_triples_1_1 && !query_triples_1_1.done && (_a = query_triples_1.return)) _a.call(query_triples_1);
            }
            finally { if (e_7) throw e_7.error; }
        }
        return bindings;
    };
    TripleStore.prototype.substituteRuleHead = function (head, binding) {
        var _a, _b, _c;
        var new_heads = [];
        var s, p, o;
        for (var result_counter = 0; result_counter < binding.len(); result_counter++) {
            if (head.s.isVar()) {
                s = (_a = binding.bindings.get(head.s.content)) === null || _a === void 0 ? void 0 : _a.at(result_counter);
            }
            else {
                s = head.s;
            }
            if (head.p.isVar()) {
                p = (_b = binding.bindings.get(head.p.content)) === null || _b === void 0 ? void 0 : _b.at(result_counter);
            }
            else {
                p = head.p;
            }
            if (head.o.isVar()) {
                o = (_c = binding.bindings.get(head.o.content)) === null || _c === void 0 ? void 0 : _c.at(result_counter);
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
        var e_8, _a, e_9, _b, e_10, _c;
        var inferred = new Array();
        var counter = 0;
        while (counter < this.triple_index.triples.length) {
            var process_triple = this.triple_index.triples.at(counter);
            if (process_triple !== undefined) {
                var matching_rules = this.rules_index.findMatch(process_triple);
                var new_triples = [];
                try {
                    for (var matching_rules_1 = (e_8 = void 0, __values(matching_rules)), matching_rules_1_1 = matching_rules_1.next(); !matching_rules_1_1.done; matching_rules_1_1 = matching_rules_1.next()) {
                        var rule = matching_rules_1_1.value;
                        var temp_bindings = this.queryWithJoin(rule.body, counter + 1);
                        var new_heads = this.substituteRuleHead(rule.head, temp_bindings);
                        try {
                            for (var new_heads_1 = (e_9 = void 0, __values(new_heads)), new_heads_1_1 = new_heads_1.next(); !new_heads_1_1.done; new_heads_1_1 = new_heads_1.next()) {
                                var new_head = new_heads_1_1.value;
                                new_triples.push(new_head);
                            }
                        }
                        catch (e_9_1) { e_9 = { error: e_9_1 }; }
                        finally {
                            try {
                                if (new_heads_1_1 && !new_heads_1_1.done && (_b = new_heads_1.return)) _b.call(new_heads_1);
                            }
                            finally { if (e_9) throw e_9.error; }
                        }
                    }
                }
                catch (e_8_1) { e_8 = { error: e_8_1 }; }
                finally {
                    try {
                        if (matching_rules_1_1 && !matching_rules_1_1.done && (_a = matching_rules_1.return)) _a.call(matching_rules_1);
                    }
                    finally { if (e_8) throw e_8.error; }
                }
                try {
                    for (var new_triples_1 = (e_10 = void 0, __values(new_triples)), new_triples_1_1 = new_triples_1.next(); !new_triples_1_1.done; new_triples_1_1 = new_triples_1.next()) {
                        var new_triple = new_triples_1_1.value;
                        // if(!this.triple_index.contains(new_triple)){
                        this.triple_index.add(new_triple);
                        inferred.push(new_triple);
                        // }
                    }
                }
                catch (e_10_1) { e_10 = { error: e_10_1 }; }
                finally {
                    try {
                        if (new_triples_1_1 && !new_triples_1_1.done && (_c = new_triples_1.return)) _c.call(new_triples_1);
                    }
                    finally { if (e_10) throw e_10.error; }
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
console.log("Call to doSomething took ".concat(endTime - startTime, " milliseconds"));
