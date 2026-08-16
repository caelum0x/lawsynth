import { placeLabel } from "../src/labels.js"; import { equal } from "./assert.js";
export function runLabelTests(): void { const placed=placeLabel({x:10,y:10,width:20,height:20},{width:10,height:5},[{x:15,y:-5,width:10,height:15}]); equal(placed.position,"right"); }
