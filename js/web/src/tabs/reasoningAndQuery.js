import {RoxiReasoner, JSBinding} from "roxi";
//import Yasqe from "@triply/yasqe";

const aboxInitialContents = "<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .";
const tboxInitialContents = "@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n {?s rdf:type test:SubClass. }=>{?s rdf:type test:SuperType.}";
const queryInitialContents = "PREFIX test: <http://www.test.be/test#>. \nPREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\nSELECT * WHERE {\n\t?s ?p ?o.\n}";


const aboxElement = document.getElementById('aboxContentQ');
const tboxElement = document.getElementById('rulesContentQ');
//const queryElement = document.getElementById('queryQ');
const reasoningShareButton = document.getElementById("shareReasoningQ");

// const yasqe = new Yasqe(
//     document.getElementById('queryQ')
// );

// aboxElement.value = aboxInitialContents;
// tboxElement.value = tboxInitialContents;
//yasqe.setValue(queryInitialContents);


const startReasoning = () => {
    const reasoner = RoxiReasoner.new();

    let startTime = performance.now();

    reasoner.add_abox(aboxInitialContents);
    reasoner.add_rules(tboxInitialContents);
    //yasqe.getValue();
    reasoner.materialize();

    let endTime = performance.now();
    let difftime = endTime-startTime ;

    let result = reasoner.query('Select * WHERE{?s ?p ?o}');
    console.log(result);
    for (const row of result){
        for(const binding of row){
            console.log("Var: " + binding.getVar() + " val: "+binding.getValue());
        }
    }

    /*
    var response={head:{vars:headVars},results:{bindings:results}};
    console.log(response);
    yasre.setResponse(response);
     */

    document.getElementById('resultsQ').value = reasoner.get_abox_dump();
    document.getElementById('timeResultsQ').innerHTML = difftime + " ms";
};

const shareReasoning = () =>{
    let host = window.location.href.split('?')[0];
    let encodedAbox = encodeURIComponent(aboxElement.value);
    let encodedRules = encodeURIComponent(tboxElement.value);
    let encodedQuery = encodeURIComponent(yasqe.getValue());

    let result = host +'?view=rq&abox='+encodedAbox+'&rules='+encodedRules+'&query='+encodedQuery;

    navigator.clipboard
        .writeText(result)
        .then(
            success => {
                reasoningShareButton.style.backgroundColor = "#43b343";
                document.getElementById("shareReasoningTextQ").style.opacity = "1";
                setTimeout(()=>{
                    reasoningShareButton.style.backgroundColor = "";
                    document.getElementById("shareReasoningTextQ").style.opacity = "0";
                }, 1000);
            },
            err => {
                reasoningShareButton.style.backgroundColor = "#e83131";
                setTimeout(()=>{
                    reasoningShareButton.style.backgroundColor = "";
                }, 1000);
                // activate share text area
                document.getElementById('shareIDQ').style.display = "block";
                // display the url
                document.getElementById('shareBoxQ').value = result;
            }
        );
}

// document.getElementById("startReasoningQ").addEventListener("click", event => {
//     startReasoning();
// });
//
// reasoningShareButton.addEventListener("click", event => {
//     shareReasoning();
// });
startReasoning();