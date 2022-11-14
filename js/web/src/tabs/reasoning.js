import {RoxiReasoner} from "roxi";

const aboxInitialContents = "<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .";
const tboxInitialContents = "@prefix test: <http://www.test.be/test#>.\n@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n{?s rdf:type test:SubClass. }=>{?s rdf:type test:SuperType.}";

const abox = document.getElementById('aboxContentR');
const tbox = document.getElementById('rulesContentR');
const reasoningShareButton = document.getElementById("shareReasoningR");

abox.value = aboxInitialContents;
tbox.value = tboxInitialContents;

const startReasoning = () => {
    const reasoner = RoxiReasoner.new();

    let startTime = performance.now();

    reasoner.add_abox(abox.value);
    reasoner.add_rules(tbox.value);
    reasoner.materialize();

    let endTime = performance.now();
    let difftime = endTime-startTime ;

    document.getElementById('resultsR').value = reasoner.get_abox_dump();
    document.getElementById('timeResultsR').innerHTML = difftime + " ms";
};

const shareReasoning = () =>{
    let host = window.location.href.split('?')[0];
    let encodedAbox = encodeURIComponent(abox.value);
    let encodedRules = encodeURIComponent(tbox.value);

    let result = host +'?view=reasoning&abox='+encodedAbox+'&rules='+encodedRules;

    navigator.clipboard
        .writeText(result)
        .then(
            success => {
                reasoningShareButton.style.backgroundColor = "#43b343";
                document.getElementById("shareReasoningTextR").style.opacity = "1";
                setTimeout(()=>{
                    reasoningShareButton.style.backgroundColor = "";
                    document.getElementById("shareReasoningTextR").style.opacity = "0";
                }, 1000);
            },
            err => {
                reasoningShareButton.style.backgroundColor = "#e83131";
                setTimeout(()=>{
                    reasoningShareButton.style.backgroundColor = "";
                }, 1000);
                // activate share text area
                document.getElementById('shareIDR').style.display = "block";
                // display the url
                document.getElementById('shareBoxR').value = result;
            }
        );
}

document.getElementById("startReasoningR").addEventListener("click", event => {
    startReasoning();
});

reasoningShareButton.addEventListener("click", event => {
    shareReasoning();
});
