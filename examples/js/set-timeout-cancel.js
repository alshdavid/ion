import console from "ion:console";
import { setTimeout, clearTimeout } from "ion:timers/timeout";

async function main() {
    for (let i = 0; i < 2; i++) {
        await waitForSetTimeout(i);
        await new Promise((res) => setTimeout(res, 2000));
    }
}

main();

async function waitForSetTimeout(run) {
    console.log(`[${run}] Sync start`);

    let int = setTimeout(() => {
        console.log(`[${run}] Should not run`);
    }, 1000);

    console.log(`[${run}] setTimeout started: ${int}`);

    setTimeout(() => {
        console.log(`[${run}] setTimeout cancelled: ${int}`);
        clearTimeout(int);
    }, 500);

    console.log(`[${run}] Sync end`);
}
