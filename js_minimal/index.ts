
class Encoder{
    decoded: Map<number, string>;
    encoded: Map<string,number>;
    counter: number;
    constructor() {
        this.counter=0;
        this.decoded= new Map<number, string>();
        this.encoded = new Map<string, number>();
    }
    add(uri: string): number{
        if (this.encoded.has(uri)){
            let val = this.encoded.get(uri);
            if(val!==undefined){return val;}
            else{
                return 0; //TODO fix
            }
        }else{
            this.encoded.set(uri,this.counter);
            this.decoded.set(this.counter, uri);
            this.counter+=1;
            return this.counter -1;
        }

    }
    decode(encoded:number): string|undefined{
        return this.decoded.get(encoded);
    }

}

interface VarOrTerm{
    content: number;
    isVar: ()=> boolean;
    isTerm: () => boolean;
}
class Var implements VarOrTerm{
    content: number;
    constructor(encoded: number) {
        this.content = encoded;
    }
    isTerm(): boolean {
        return false;
    }

    isVar(): boolean {
        return true;
    }
}
class Term implements VarOrTerm{
    content: number;
    constructor(encoded: number) {
        this.content = encoded;
    }
    isTerm(): boolean {
        return true;
    }

    isVar(): boolean {
        return false;
    }
}

class Triple{
    s: VarOrTerm;
    p: VarOrTerm;
    o: VarOrTerm;
    constructor() {
    }
    static from(s_string:string, p_string: string, o_string: string, encoder:Encoder) : Triple{
        let new_triple =new Triple();
        new_triple.s=this.createVarOrTerm(s_string,encoder);
        new_triple.p=this.createVarOrTerm(p_string,encoder);
        new_triple.o=this.createVarOrTerm(o_string,encoder);
        return new_triple
    }
    static tripleFromEncoded(s_term:VarOrTerm, p_term: VarOrTerm, o_term: VarOrTerm) {
        let new_triple =new Triple();
        new_triple.s = s_term;
        new_triple.o = o_term;
        new_triple.p = p_term;
        return new_triple;

    }
    static createVarOrTerm(item: string, encoder: Encoder){
        if(item.startsWith("?")){
            return new Var(encoder.add(item));
        }else{
            return new Term(encoder.add(item));
        }
    }
}
class Rule{
    head: Triple;
    body: Triple[];
    constructor(head:Triple, body:Triple[]) {
        this.head = head;
        this.body = body;
    }
}
class TripleIndex{
    triples: Triple[];
    spo:Map<number,Map<number,[number,number][]>>;
    pos:Map<number,Map<number,[number,number][]>>;
    osp:Map<number,Map<number,[number,number][]>>;
    counter: number;
    constructor() {
        this.triples = [];
        this.spo = new Map<number, Map<number, [number,number][]>>();
        this.pos = new Map<number, Map<number, [number,number][]>>();
        this.osp = new Map<number, Map<number, [number,number][]>>();
        this.counter = 0;
    }

    len() :number{
        return this.triples.length;
    }
    add(triple: Triple){
        //spo
        if (! this.spo.has(triple.s.content)){
            this.spo.set(triple.s.content, new Map<number,[number,number][]>());
        }
        if(! this.spo.get(triple.s.content)?.has(triple.p.content)){
            this.spo.get(triple.s.content)?.set(triple.p.content, []);
        }
        this.spo.get(triple.s.content)?.get(triple.p.content)?.push([triple.o.content,this.counter]);
        //pos
        if (! this.pos.has(triple.p.content)){
            this.pos.set(triple.p.content, new Map<number,[number,number][]>());
        }
        if(! this.pos.get(triple.p.content)?.has(triple.o.content)){
            this.pos.get(triple.p.content)?.set(triple.o.content, []);
        }
        this.pos.get(triple.p.content)?.get(triple.o.content)?.push([triple.s.content,this.counter]);
        // osp
        if (! this.osp.has(triple.o.content)){
            this.osp.set(triple.o.content, new Map<number,[number,number][]>());
        }
        if(! this.osp.get(triple.o.content)?.has(triple.s.content)){
            this.osp.get(triple.o.content)?.set(triple.s.content, []);
        }
        this.osp.get(triple.o.content)?.get(triple.s.content)?.push([triple.p.content,this.counter]);
        this.triples.push(triple);
    }
    contains(triple:Triple):boolean{
        if (! this.osp.has(triple.o.content)){
            return false;
        }else{
            if(! this.osp.get(triple.o.content)?.has(triple.s.content)){
                return false;
            }else{
                // @ts-ignore
                //return this.osp.get(triple.o.content)?.get(triple.s.content)?.includes(triple.p.content);
                for (let [encoded, counter] of this.osp.get(triple.o.content)?.get(triple.s.content)){
                    if(encoded == triple.p.content){
                        return true;
                    }
                }
                return false;
            }
        }

    }
    query(query_triple: Triple, triple_counter:number|undefined): Binding{
        let matched_binding = new Binding();
        let counter_check = triple_counter ?? this.counter;
        //?s p o
        if(query_triple.s.isVar() && query_triple.p.isTerm() && query_triple.o.isTerm()){
            let indexes = this.pos.get(query_triple.p.content);
            if (!!indexes){
                let indexes2 = indexes.get(query_triple.o.content);
                if(!!indexes2){

                    for(let [encoded, counter] of indexes2){

                        if(counter <= counter_check){
                            matched_binding.add(query_triple.s.content,encoded);
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //s ?p o
        else   if(query_triple.s.isTerm() && query_triple.p.isVar() && query_triple.o.isTerm()){
            let indexes = this.osp.get(query_triple.o.content);
            if (!!indexes){
                let indexes2 = indexes.get(query_triple.s.content);
                if(!!indexes2){
                    for(let [encoded, counter] of indexes2){
                        if(counter <= counter_check){
                            matched_binding.add(query_triple.p.content,encoded);
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        // s p ?o
        else if(query_triple.s.isTerm() && query_triple.p.isTerm() && query_triple.o.isVar()){
            let indexes = this.spo.get(query_triple.p.content);
            if (!!indexes){
                let indexes2 = indexes.get(query_triple.p.content);
                if(!!indexes2){
                    for(let [encoded, counter] of indexes2){
                        if(counter <= counter_check){
                            matched_binding.add(query_triple.o.content,encoded);
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //?s ?p o
        else  if(query_triple.s.isVar() && query_triple.p.isVar() && query_triple.o.isTerm()){
            let indexes = this.osp.get(query_triple.o.content);
            if (!!indexes){
                for(let [s_key, p_values] of indexes){
                    for(let [encoded, counter] of p_values){
                        if(counter <= counter_check){
                            matched_binding.add(query_triple.s.content,s_key);
                            matched_binding.add(query_triple.p.content,encoded);
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //s ?p ?o
        else  if(query_triple.s.isTerm() && query_triple.p.isVar() && query_triple.o.isVar()){
            let indexes = this.spo.get(query_triple.s.content);
            if (!!indexes){
                for(let [p_key, o_values] of indexes){
                    for(let [encoded, counter] of o_values){
                        if(counter <= counter_check){
                            matched_binding.add(query_triple.p.content,p_key);
                            matched_binding.add(query_triple.o.content,encoded);
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //?s p ?o
        else  if(query_triple.s.isVar() && query_triple.p.isTerm() && query_triple.o.isVar()){
            let indexes = this.pos.get(query_triple.p.content);
            if (!!indexes){
                for(let [o_key, s_values] of indexes){
                    for(let [encoded, counter] of s_values){
                        if(counter <= counter_check){
                            matched_binding.add(query_triple.o.content,o_key);
                            matched_binding.add(query_triple.s.content,encoded);
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        //?s ?p ?o
        else  if(query_triple.s.isVar() && query_triple.p.isVar() && query_triple.o.isVar()){
            for (let [s_key, p_values] of this.spo){
                for(let [p_key, o_values] of p_values){
                    for(let [encoded, counter] of o_values){
                        if(counter <= counter_check){
                            matched_binding.add(query_triple.s.content,s_key);
                            matched_binding.add(query_triple.p.content,p_key);
                            matched_binding.add(query_triple.o.content,encoded);
                        }else{
                            break;
                        }
                    }
                }
            }
        }
        return matched_binding;

    }
}
class RuleIndex{
    spo: Rule[];
    s:Map<number,Rule[]>;
    p:Map<number,Rule[]>;
    o:Map<number,Rule[]>;
    sp:Map<number,Map<number,Rule[]>>;
    po:Map<number,Map<number,Rule[]>>;
    so:Map<number,Map<number,Rule[]>>;
    constructor() {
        this.s = new Map<number, Rule[]>();
        this.o = new Map<number, Rule[]>();
        this.p = new Map<number, Rule[]>();
        this.sp = new Map<number, Map<number, Rule[]>>();
        this.po= new Map<number, Map<number, Rule[]>>();
        this.so = new Map<number, Map<number, Rule[]>>();
        this.spo = [];
    }
    len():number{
        return this.spo.length;
    }
    add(rule:Rule){
        rule.body.forEach((triple)=>{
            let s = triple.s;
            let o = triple.o;
            let p = triple.p;
            //s match
            if(s.isTerm() && p.isVar() && o.isVar()){
                if(!this.s.has(s.content)){
                    this.s.set(s.content,[]);
                }
                if(!this.s.get(s.content)?.includes(rule)){
                    this.s.get(s.content)?.push(rule);
                }
            }
            //p match
            if(s.isVar() && p.isTerm() && o.isVar()){
                if(!this.p.has(p.content)){
                    this.p.set(p.content,[]);
                }
                if(!this.p.get(p.content)?.includes(rule)){
                    this.p.get(p.content)?.push(rule);
                }
            }
            //o match
            if(s.isVar() && p.isVar() && o.isTerm()){
                if(!this.o.has(s.content)){
                    this.o.set(s.content,[]);
                }
                if(!this.o.get(s.content)?.includes(rule)){
                    this.o.get(s.content)?.push(rule);
                }
            }
            //sp match
            if(s.isTerm() && p.isTerm() && o.isVar()){
                if(!this.sp.has(s.content)){
                    this.sp.set(s.content,new Map<number,[]>());
                }
                if(!this.sp.get(s.content)?.has(p.content)){
                    this.sp.get(s.content)?.set(p.content,[]);
                }
                if(!this.sp.get(s.content)?.get(p.content)?.includes(rule)){
                    this.sp.get(s.content)?.get(p.content)?.push(rule);
                }
            }
            //so match
            if(s.isTerm() && p.isVar() && o.isTerm()){
                if(!this.so.has(s.content)){
                    this.so.set(s.content,new Map<number,[]>());
                }
                if(!this.so.get(s.content)?.has(o.content)){
                    this.so.get(s.content)?.set(o.content,[]);
                }
                if(!this.so.get(s.content)?.get(o.content)?.includes(rule)){
                    this.so.get(s.content)?.get(o.content)?.push(rule);
                }
            }
            //po match
            if(s.isVar() && p.isTerm() && o.isTerm()){
                if(!this.po.has(p.content)){
                    this.po.set(p.content,new Map<number,[]>());
                }
                if(!this.po.get(p.content)?.has(o.content)){
                    this.po.get(p.content)?.set(o.content,[]);
                }
                if(!this.po.get(p.content)?.get(o.content)?.includes(rule)){
                    this.po.get(p.content)?.get(o.content)?.push(rule);
                }
            }
            //spo
            if(s.isVar() && p.isVar() && o.isVar()){
                if(!this.spo.includes(rule)){
                    this.spo.push(rule);
                }
            }
        });
    }
    findMatch(triple:Triple): Rule[]{
        let matchedTripels: Rule[] = [];
        //check s
        if (this.s.has(triple.s.content)){
            this.s.get(triple.s.content)?.forEach((t)=>matchedTripels.push(t));
        }
        //check p
        if (this.p.has(triple.p.content)){
            this.p.get(triple.p.content)?.forEach((t)=>matchedTripels.push(t));
        }
        //check o
        if (this.o.has(triple.o.content)){
            this.o.get(triple.o.content)?.forEach((t)=>matchedTripels.push(t));
        }
        //check sp
        if(this.sp.has(triple.s.content)){
            if(this.sp.get(triple.s.content)?.has(triple.p.content)){
                this.sp.get(triple.s.content)?.get(triple.p.content)?.forEach((t)=>matchedTripels.push(t));
            }
        }
        //check so
        if(this.so.has(triple.s.content)){
            if(this.so.get(triple.s.content)?.has(triple.o.content)){
                this.so.get(triple.s.content)?.get(triple.o.content)?.forEach((t)=>matchedTripels.push(t));
            }
        }
        //check po
        if(this.po.has(triple.p.content)){
            if(this.po.get(triple.p.content)?.has(triple.o.content)){

                this.po.get(triple.p.content)?.get(triple.o.content)?.forEach((t)=>matchedTripels.push(t));

            }
        }
        this.spo.forEach((t)=>matchedTripels.push(t));
        return matchedTripels;
    }
}
class Binding{
    bindings : Map<number, number[]>;
    constructor() {
        this.bindings = new Map<number, number[]>();
    }
    add(var_name: number, term:number){
        if(!this.bindings.has(var_name)){
            this.bindings.set(var_name,[]);
        }
        this.bindings.get(var_name)?.push(term);
    }
    len():number{
        for (const [key, value] of this.bindings.entries()){
            return value.length;
        }
        return 0;
    }
    join(join_binding: Binding): Binding{
        let left = join_binding;
        left = this;
        let right = join_binding;
        if(left.len()==0){return right;}
        if(right.len()==0){return left;}
        let result = new Binding();
        let left_bindings = left.bindings;
        let right_bindings = right.bindings;
        if (left.len()<right.len()){
            right_bindings = this.bindings;
            left = join_binding;
        }
        let join_keys = [];
        for( const key of left.bindings.keys()){
            if(right.bindings.has(key)){
                join_keys.push(key);
            }
        }
        for (let left_c =0 ; left_c<left.len(); left_c++){
            for( let right_c = 0; right_c < right.len(); right_c++){
                let match_keys = true;
                for(let join_key of join_keys){
                    let left_term = left.bindings.get(join_key)?.at(left_c);
                    let right_term = right.bindings.get(join_key)?.at(right_c);
                    if (left_term != right_term){
                        match_keys = false;
                        break;
                    }
                }
                if(match_keys){
                    for( const key of left.bindings.keys()){
                        // @ts-ignore
                        result.add(key, left.bindings.get(key)?.at(left_c));
                    }
                    for( const key of right.bindings.keys()){
                        if(!left.bindings.has(key)){
                            // @ts-ignore
                            result.add(key, right.bindings.get(key)?.at(right_c));
                        }

                    }
                }
            }
        }

        return result;

    }
}
class TripleStore{
    rules_index: RuleIndex;
    rules: Rule[];
    triple_index: TripleIndex;
    encoder: Encoder;
    constructor() {
        this.rules_index = new RuleIndex();
        this.rules = [];
        this.triple_index = new TripleIndex();
        this.encoder = new Encoder();
    }
    add(triple:Triple){
        this.triple_index.add(triple);
    }
    add_rule(rule:Rule){
        this.rules_index.add(rule);
        this.rules.push(rule);
    }
    query(query_triple:Triple, triple_counter: number|undefined): Binding{
        let bindings = new Binding();
        let counter = triple_counter ??  this.triple_index.len();
        for(let triple of this.triple_index.triples.slice(0,counter)){
            if(query_triple.s.isVar()){
                bindings.add(query_triple.s.content,triple.s.content);
            }else{
                if(query_triple.s.content!= triple.s.content){
                    break;
                }
            }
            if(query_triple.p.isVar()){
                bindings.add(query_triple.p.content,triple.p.content);
            }else{
                if(query_triple.p.content!= triple.p.content){
                    break;
                }
            }
            if(query_triple.o.isVar()){
                bindings.add(query_triple.o.content,triple.o.content);
            }else{
                if(query_triple.o.content!= triple.o.content){
                    break;
                }
            }
        }

        return bindings;
    }
    queryWithJoin(query_triples:Triple[], triple_counter: number | undefined): Binding{
        let bindings = new Binding();
        for( let query_triple of query_triples){
            //let current_binding = this.query(query_triple,triple_counter);
            let current_binding = this.triple_index.query(query_triple,triple_counter);
            bindings = bindings.join(current_binding);
        }
        return bindings;
    }
    substituteRuleHead(head:Triple, binding:Binding): Triple[]{
        let new_heads = [];
        let s,p,o;
        for(let result_counter = 0 ; result_counter < binding.len(); result_counter++){
            if(head.s.isVar()){
                // @ts-ignore
                s = new Term(binding.bindings.get(head.s.content)?.at(result_counter));

            }else{
                s= head.s;
            }
            if(head.p.isVar()){
                // @ts-ignore
                p = new Term(binding.bindings.get(head.p.content)?.at(result_counter));
            }else{
                p= head.p;
            }
            if(head.o.isVar()){
                // @ts-ignore
                o = new Term(binding.bindings.get(head.o.content)?.at(result_counter));
            }else{
                o= head.o;
            }

            // @ts-ignore
            new_heads.push(Triple.tripleFromEncoded(s,p,o));
        }
        return new_heads;
    }
    materialize(): Triple[]{
        let inferred = new Array<Triple>();
        let counter = 0;
        while(counter < this.triple_index.triples.length){
            let process_triple = this.triple_index.triples.at(counter);
            if(process_triple!== undefined){
                let matching_rules = this.rules_index.findMatch(process_triple);
                let new_triples = [];
                for(let rule of matching_rules){
                    let temp_bindings = this.queryWithJoin(rule.body,counter + 1);
                    let new_heads = this.substituteRuleHead(rule.head,temp_bindings);

                    for (let new_head of new_heads){
                        new_triples.push(new_head);
                    }
                }
                for (let new_triple of new_triples){
                     if(!this.triple_index.contains(new_triple)){
                         this.triple_index.add(new_triple);
                         inferred.push(new_triple);
                     }
                }
            }
            counter+=1;
        }

        return inferred;
    }

    static parse_triple(data: string, encoder: Encoder): Triple{
        let items = data.trim().split(" ");
        let s = Triple.createVarOrTerm(items[0].trim(), encoder);
        let p = Triple.createVarOrTerm(items[1].trim(), encoder);
        console.log(items);
        let o = (items[2].trim().endsWith('.'))? Triple.createVarOrTerm(items[2].trim().slice(0,-1), encoder)
            :Triple.createVarOrTerm(items[2].trim(), encoder);
        return  Triple.tripleFromEncoded(s,p,o);

    }
    static remove_first_and_last(value:string): string{
        return value.slice(1,-1);
    }
    static parse(data: string, encoder: Encoder): [Triple[],Rule[]]{
        let rules = new Array<Rule>();
        let content = new Array<Triple>();
        for(let line of data.split("\n")){
            if(line.includes("=>")) {
                let rule = line.split("=>");
                let body = this.remove_first_and_last(rule[0].trim());
                let head = this.remove_first_and_last(rule[1].trim());
                let head_triple = this.parse_triple(head, encoder);
                let body_triples = new Array<Triple>();
                for (let body_triple of body.split(".")) {
                    body_triples.push(this.parse_triple(body_triple, encoder));
                }
                rules.push(new Rule(head_triple, body_triples));
            }else{
                let triple = this.parse_triple(line,encoder);
                content.push(triple);
            }
        }
        return [content,rules];
    }
}
let encoder = new Encoder();
//parse test

let data=":a a :A.\n\
        :b a :B.\n\
        {?a a :A}=>{?a a :C}\n\
        {?a a :B}=>{?a a :D}";
let [content, rules] = TripleStore.parse(data, encoder);

console.log("Content {:?}", content);
console.log("Rules {:?}", rules);
console.log("encoded {:?}", encoder.decoded);

let triple_store = new TripleStore();
content.forEach(function(triple){triple_store.add(triple)});
rules.forEach(function(rule){triple_store.add_rule(rule)});

// for(let i = 0; i < 1; i++) {
//     triple_store.add(Triple.from("s"+i,"p","o0",encoder));
//
//     let triple_head = Triple.from("?s1", "p", "o"+(i+1), encoder);
//     let triple_body1 = Triple.from("?s1", "p", "o"+i, encoder);
//     let rule = new Rule(triple_head, [triple_body1]);
//     triple_store.add_rule(rule);
//     let triple_head2 = Triple.from("?s1", "p", "i"+(i+1), encoder);
//     let triple_body21 = Triple.from("?s1", "p", "o"+i, encoder);
//     let rule2 = new Rule(triple_head2, [triple_body21]);
//     triple_store.add_rule(rule2);
//     let triple_head3 = Triple.from("?s1", "p", "j"+(i+1), encoder);
//     let triple_body31 = Triple.from("?s1", "p", "o"+i, encoder);
//     let rule3 = new Rule(triple_head3, [triple_body31]);
//     triple_store.add_rule(rule3);
// }
var startTime = performance.now();

let inferred  = triple_store.materialize();
console.log("inferred");
console.log(inferred.length);



var endTime = performance.now();

console.log(`Call to doSomething took ${endTime - startTime} milliseconds`);
