require("./tabs/reasoning");
require("./tabs/rsp");
require("./tabs/reasoningAndQuery");
const {yasqeRSP} = require("./tabs/rsp");
const {yasqeQ} = require("./tabs/reasoningAndQuery");

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
    if( view === 'reasoning'){
        openTab(event,'reasoning');
        decodeAndAssign(gup('abox', window.location.href),'aboxContentR');
        decodeAndAssign(gup('rules', window.location.href),'rulesContentR');

    }
    else if (view === 'rsp') {
        openTab(event,'rsp');
        decodeAndAssign(gup('rules', window.location.href),'rulesContentRSP');
        decodeAndAssign(gup('windowWidth', window.location.href),'windowWidth');
        decodeAndAssign(gup('windowSlide', window.location.href),'windowSlide');
        decodeAndAssign(gup('eventID', window.location.href),'eventID');
        decodeAndAssign(gup('timestamp', window.location.href),'timestamp');
        try {
            let decoded = decodeURIComponent(gup('query', window.location.href));
            yasqeRSP.setValue(decoded);
        } catch (e) {
            console.error(e);
        }
    }
    else if(view === 'rq') {
        openTab(event,'rq');
        decodeAndAssign(gup('abox', window.location.href),'aboxContentQ');
        decodeAndAssign(gup('rules', window.location.href),'rulesContentQ');
        try {
            let decoded = decodeURIComponent(gup('query', window.location.href));
            yasqeQ.setValue(decoded);
        } catch (e) {
            console.error(e);
        }
    }
    else {
        openTab(event,'reasoning');
        decodeAndAssign(gup('abox', window.location.href),'aboxContentR');
        decodeAndAssign(gup('rules', window.location.href),'rulesContentR');
    }
}
start();
