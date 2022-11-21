import {RoxiReasoner, JSBinding} from "roxi";
import Yasqe from "@triply/yasqe";
import Yasr from "@triply/yasr";

const aboxInitialContents = "<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .";
const tboxInitialContents = "@prefix test: <http://www.test.be/test#>.\n@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n{?s rdf:type test:SubClass. }=>{?s rdf:type test:SuperType.}";
const queryInitialContents = "SELECT * WHERE {\n\t?s ?p ?o.\n}";


const aboxElement = document.getElementById('aboxContentQ');
const tboxElement = document.getElementById('rulesContentQ');
const reasoningSwitch = document.getElementById("reasoningSwitchQ");
const reasoningShareButton = document.getElementById("shareReasoningQ");

export const yasqeQ = new Yasqe(
    document.getElementById('queryQ')
);

const yasr = new Yasr(
    document.getElementById('resultsQ')
);

yasr.setResponse({head:{vars:[""]},results:{bindings:[{"":{type:"literal",value: ""}}]}});

reasoningSwitch.checked = true;
aboxElement.value = aboxInitialContents;
tboxElement.value = tboxInitialContents;
yasqeQ.setValue(queryInitialContents);

const urlRegex = new RegExp(/<?(https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&\/\/=]*))>?/);

const startReasoning = () => {
    const reasoner = RoxiReasoner.new();

    const startTime = performance.now();

    reasoner.add_abox(aboxElement.value);
    reasoner.add_rules(tboxElement.value);

    if (reasoningSwitch.checked) {
        reasoner.materialize();
    }

    const endTime = performance.now();
    const difftime = endTime-startTime ;

    const result = reasoner.query(yasqeQ.getValue().toString());
    const results = [];
    let temp = {};
    let headVars = new Map();
    for (const row of result){
        temp = {};
        for(const binding of row){
            headVars.set(binding.getVar(), binding.getVar());
            const regexArray = urlRegex.exec(binding.getValue());
            if (regexArray == null) {
                temp[binding.getVar()] = {type:"literal",value: binding.getValue()};
            }
            else {
                temp[binding.getVar()] = {type:"uri",value: regexArray[1]};
            }
        }
        results.push(temp)
    }
    const response={head:{vars:Array.from(headVars.keys())},results:{bindings:results}};
    yasr.setResponse(response);
    document.getElementById('timeResultsQ').innerHTML = difftime + " ms";
};

const shareReasoning = () =>{
    let host = window.location.href.split('?')[0];
    let encodedAbox = encodeURIComponent(aboxElement.value);
    let encodedRules = encodeURIComponent(tboxElement.value);
    let encodedQuery = encodeURIComponent(yasqeQ.getValue());

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

document.getElementById("startReasoningQ").addEventListener("click", event => {
    startReasoning();
});

reasoningShareButton.addEventListener("click", event => {
    shareReasoning();
});
