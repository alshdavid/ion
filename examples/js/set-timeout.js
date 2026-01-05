import console from "ion:console";
import { setTimeout } from "ion:timers/timeout";

console.log("Sync start");

setTimeout(() => console.log("Async done"), 1000);

console.log("Sync end");
