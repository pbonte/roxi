import {JSRSPEngine, RoxiReasoner, JSBinding} from "roxi";

let abox= "<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .";
let tbox = "@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n {?s rdf:type test:SubClass. }=>{?s rdf:type test:SuperType.}"
let rsp_rules = "@prefix test: <http://test/>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n {?x test:isIn ?y. ?y test:isIn ?z. }=>{?x test:isIn ?z.}"
console.log(encodeURIComponent(abox));
document.getElementById('aboxContent').value = abox;
document.getElementById('rulesContent').value = tbox;
document.getElementById('rulesContentRSP').value = rsp_rules;
document.getElementById('rulesContentRSP').value = rsp_rules;
document.getElementById('continousQuery').value = "Select * WHERE{ ?x <http://test/isIn> ?y}";

const reasoningButton = document.getElementById("startReasoning");
const reasoningShareButton = document.getElementById("shareReasoning");

const rspButton = document.getElementById("startRSP");
var currentTs = 0;
var rspEngine = null;

function gup( name, url ) {
    if (!url) url = location.href;
    name = name.replace(/[\[]/,"\\\[").replace(/[\]]/,"\\\]");
    var regexS = "[\\?&]"+name+"=([^&#]*)";
    var regex = new RegExp( regexS );
    var results = regex.exec( url );
    return results == null ? null : results[1];
}
function decodeAndAssign(toDecode,elementID){
    if(toDecode){
        try {
            let decoded = decodeURIComponent(toDecode);
            document.getElementById(elementID).value = decoded;
        } catch (e) {
            console.error(e);
        }
    }
}
function start(){
    let view = gup('view', window.location.href);
    if( view != 'rsp'){
        openTab(event,'reasoning')
        let abox = gup('abox', window.location.href);
        decodeAndAssign(abox,'aboxContent');

        let rules = gup('rules', window.location.href);
        decodeAndAssign(rules,'rulesContent');

    }else{
        openTab(event,'rsp')

    }
}
start();

const startReasoning = () => {

    const reasoner = RoxiReasoner.new();

    let abox_new= document.getElementById('aboxContent').value;
    let tbox_new = document.getElementById('rulesContent').value;
    let startTime = new Date();

    reasoner.add_abox(abox_new);
    reasoner.add_rules(tbox_new);
    reasoner.materialize();
    let endTime = new Date();
    let difftime = endTime-startTime ;
    document.getElementById('results').value = reasoner.get_abox_dump();
    document.getElementById('timeResults').innerHTML = difftime + " ms";

};


reasoningButton.addEventListener("click", event => {
    startReasoning();
});
rspButton.addEventListener("click", event => {
    startRSP();
});
reasoningShareButton.addEventListener("click", event => {
    shareReasoning();
});

// callback function
function callback(val) {
    val.forEach((x, i) => console.log(x.toString()));
    document.getElementById('resultsRSP').value = val +"@"+currentTs+"\n" + document.getElementById('resultsRSP').value;

}

const startRSP = () => {
    if(rspEngine == null){
        console.log("starting");
        let tbox_new = document.getElementById('rulesContentRSP').value;
        document.getElementById('rulesContentRSP').setAttribute('disabled', '');

        let abox = "";
        let query = document.getElementById('continousQuery').value;
        document.getElementById('continousQuery').setAttribute('disabled', '');

        let width = document.getElementById('windowWidth').value;
        document.getElementById('windowWidth').setAttribute('disabled', '');
        let slide = document.getElementById('windowSlide').value;
        document.getElementById('windowSlide').setAttribute('disabled', '');
        rspEngine = JSRSPEngine.new(width,slide,tbox_new,abox,query,callback);
    }
    currentTs+=1;
    let event = document.getElementById('eventID').value;
    rspEngine.add(event, currentTs);
    document.getElementById('timestamp').value = currentTs;
    document.getElementById('eventID').value = "<http://test/"+currentTs+"> <http://test/isIn> <http://test/"+(currentTs+1)+">.";


    console.log("stopped");
}

const shareReasoning = () =>{
    let host = window.location.href.split('?')[0];
    let encodedAbox = encodeURIComponent(document.getElementById('aboxContent').value);
    let encodedRules = encodeURIComponent(document.getElementById('rulesContent').value);

    let result = host +'?view=reasoning&abox='+encodedAbox+'&rules='+encodedRules;
    // activate share text area
    document.getElementById('shareID').style.display = "block";
    // display the url
    document.getElementById('shareBox').value = result;

}