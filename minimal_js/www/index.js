import {RustReasoner} from "rustreasoner";


function start_experiment(content) {
    let startTime_load = new Date();
    const reasoner = RustReasoner.from(content);
    let endTime_load = new Date();
    let load_time = endTime_load - startTime_load;
    console.log("Load:\t" + load_time + " ms");

    let startTime = new Date();
    reasoner.materialize();
    let endTime = new Date();
    let elapsed = endTime - startTime;
    console.log("Reasoning:\t" + elapsed + " ms");
    let startTime_extract = new Date();
    let output = reasoner.content_to_string();
    let endTime_extract = new Date();
    console.log("Content Length:\t" + output.length);
    console.log("Num results:\t" + reasoner.len());

    let extract_time = endTime_extract - startTime_extract;
    console.log("Extraction:\t" + extract_time + " ms");
    console.log("{\"loadtime\": "+load_time+", \"processtime\": "+elapsed+", \"extracttime\": "+extract_time + ", \"depth\": 10, \"num_inds\": 100,  \"mode\": \"wasm_eval2\" }");

}

const startHierarchyReasoning = () => {
    let max_depth = 1000000;
    let num_iters = 2;
    for(let iter = 0; iter < num_iters; iter++) {
        let content = ":a a :C0.\n";
        for (let i = 0; i < max_depth; i++) {
            content += "{?a a :C" + i + "}=>{?a a :C" + (i + 1) + "}\n";
            content += "{?a a :C" + i + "}=>{?a a :N" + (i + 1) + "}\n";
            content += "{?a a :C" + i + "}=>{?a a :J" + (i + 1) + "}\n";
        }
        content += "{?a a :C" + (max_depth) + "}=>{?a a :C" + (max_depth + 1) + "}";

        start_experiment(content);
    }

};



const startJoinReasoning = () => {
    let max_depth = 25;
    let num_individuals = 100;
    let num_iters = 2;
    for(let iter = 0; iter < num_iters; iter++) {
        let content = "";
        for(let ind_it = 0 ; ind_it < num_individuals; ind_it++){
            content += ":a"+ind_it+" a :C0.\n";
            content += ":a"+ind_it+" a :C1.\n";
        }
        for (let i = 0; i < max_depth; i++) {
            content += "{?a a :C" + i + ".?a a :C"+(i+1)+" }=>{?a a :C" + (i + 2) + "}\n";
        }
        content = content.slice(0,-1);

        start_experiment(content);
    }

};


startJoinReasoning();
