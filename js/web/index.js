import {RoxiReasoner} from "roxi";


let abox= "<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .";
let tbox = "@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n {?s rdf:type test:SubClass. }=>{?s rdf:type test:SuperType.}"
document.getElementById('aboxContent').value = abox;
document.getElementById('rulesContent').value = tbox;



const reasoningButton = document.getElementById("startReasoning");

const startReasoning = () => {
    const reasoner = RoxiReasoner.new();

    let abox_new= document.getElementById('aboxContent').value;
    let tbox_new = document.getElementById('rulesContent').value;

    reasoner.add_abox(abox_new);
    reasoner.add_rules(tbox_new);
    reasoner.materialize();

    document.getElementById('results').value = reasoner.get_abox_dump();
};



reasoningButton.addEventListener("click", event => {
    startReasoning();
});