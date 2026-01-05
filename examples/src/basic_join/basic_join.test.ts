import { executeExample } from "../../test-utils/run_test.ts";
import { assert, assertEquals, assertObjectMatch } from "jsr:@std/assert@^1";

type Record = {
    thread: number;
    message: string;
    js_context?: number;
    event_loop?: boolean;
};

async function executeBasicJoin(caseName: string): Promise<Array<Record>> {
    const result = await executeExample("basic_join", [caseName]);
    return result.split("\n").map((record) => JSON.parse(record));
}

function assertArraysMatch(arr1: any, arr2: any, msg?: string): void {
    return assertObjectMatch({ arr: arr1 }, { arr: arr2 }, msg);
}

function filterJs(ctx: number, event_loop: boolean = false) {
    return (r: Record): boolean =>
        r.js_context === ctx && !!r.event_loop == event_loop;
}


Deno.test("should_cancel_when_dropped", async () => {
    const example = "should_cancel_when_dropped";
    const results = await executeBasicJoin(example);

    // The code on the main thread will always run
    assertEquals(results.filter((r) => r.thread === 1).length, 2);

    // The code on the JavaScript thread may or may not run
    assert(
        results.filter((r) => r.js_context === 0 && !r.event_loop).length ===
            2 ||
            results.filter((r) => r.js_context === 0 && !r.event_loop)
                .length === 0
    );

    // The code on the Event Loop should not run
    assertEquals(results.filter((r) => r.event_loop).length, 0);
});

Deno.test("should_cancel_when_dropped_multiple", async () => {
    const example = "should_cancel_when_dropped_multiple";
    const results = await executeBasicJoin(example);

    // The code on the main thread will always run
    assertEquals(results.filter((r) => r.thread === 1).length, 2);

    // The code on the JavaScript thread may or may not run
    assert(
        results.filter((r) => r.js_context === 0 && !r.event_loop).length ===
            4 ||
            results.filter((r) => r.js_context === 0 && !r.event_loop)
                .length === 0
    );

    // The code on the Event Loop should not run
    assertEquals(results.filter((r) => r.event_loop).length, 0);
});

Deno.test("should_cancel_blocking_when_dropped", async () => {
    const example = "should_cancel_blocking_when_dropped";
    const results = await executeBasicJoin(example);

    // The code on the main thread will always run
    assertEquals(results.filter((r) => r.thread === 1).length, 2);

    // The code on the JavaScript thread must run
    assertEquals(
        results.filter((r) => r.js_context === 0 && !r.event_loop).length,
        2
    );

    // The code on the Event Loop may or may not run, but not progress
    assert(
        results.filter(
            (r) => r.js_context === 0 && r.event_loop && r.message !== "end"
        ).length === 1 ||
            results.filter((r) => r.js_context === 0 && r.event_loop).length ===
                0
    );
});

Deno.test("should_cancel_blocking_when_dropped_multiple", async () => {
    const example = "should_cancel_blocking_when_dropped_multiple";
    const results = await executeBasicJoin(example);

    // The code on the main thread will always run
    assertEquals(results.filter((r) => r.thread === 1).length, 2);

    // The code on the JavaScript thread must run
    assertEquals(
        results.filter((r) => r.js_context === 0 && !r.event_loop).length,
        4
    );

    // The code on the Event Loop may or may not run, but not progress
    assert(
        results.filter(
            (r) => r.js_context === 0 && r.event_loop && r.message !== "end"
        ).length === 2 ||
            results.filter((r) => r.js_context === 0 && r.event_loop).length ===
                0
    );
});

Deno.test("should_wait_for_code_to_finish", async () => {
    const example = "should_wait_for_code_to_finish";
    const results = await executeBasicJoin(example);
    assertArraysMatch(results, [
        { thread: 1, message: "start" },
        { thread: 2, js_context: 0, message: "start" },
        { thread: 2, js_context: 0, message: "end" },
        { thread: 1000, js_context: 0, event_loop: true, message: "start" },
        { thread: 1000, js_context: 0, event_loop: true, message: "end" },
        { thread: 2, js_context: 0, message: "resolved" },
        { thread: 1, message: "end" },
    ]);
});

// Hangs
Deno.test.only("should_wait_for_code_to_finish_multiple", async () => {
    const example = "should_wait_for_code_to_finish_multiple";
    const results = await executeBasicJoin(example);

    console.log(results);

    assertArraysMatch(results, [
        { thread: 1, message: "start" },
        { thread: 2, js_context: 0, message: "start" },
        { thread: 2, js_context: 0, message: "end" },
        { thread: 1000, js_context: 0, event_loop: true, message: "start" },
        { thread: 1000, js_context: 0, event_loop: true, message: "end" },
        { thread: 2, js_context: 0, message: "resolved" },
        { thread: 1, message: "end" },
    ]);
});

Deno.test("should_wait_for_code_to_finish_blocking", async () => {
    const example = "should_wait_for_code_to_finish_blocking";
    const results = await executeBasicJoin(example);
    assertArraysMatch(results, [
        { thread: 1, message: "start" },
        { thread: 2, js_context: 0, message: "start" },
        { thread: 2, js_context: 0, message: "end" },
        { thread: 1000, js_context: 0, event_loop: true, message: "start" },
        { thread: 1000, js_context: 0, event_loop: true, message: "end" },
        { thread: 2, js_context: 0, message: "resolved" },
        { thread: 1, message: "end" },
    ]);
});

// Does not complete context
Deno.test.ignore("should_wait_for_code_to_finish_worker", async () => {
    const example = "should_wait_for_code_to_finish_worker";
    const results = await executeBasicJoin(example);
    assertArraysMatch(results, [
        { thread: 1, message: "start" },
        { thread: 2, js_context: 0, message: "start" },
        { thread: 2, js_context: 0, message: "end" },
        { thread: 1000, js_context: 0, event_loop: true, message: "start" },
        { thread: 1000, js_context: 0, event_loop: true, message: "end" },
        { thread: 2, js_context: 0, message: "resolved" },
        { thread: 1, message: "end" },
    ]);
});

// Does not complete context
Deno.test.ignore("should_wait_for_code_to_finish_worker_blocking", async () => {
    const example = "should_wait_for_code_to_finish_worker_blocking";
    const results = await executeBasicJoin(example);

    console.log(results);

    assertArraysMatch(results, [
        { thread: 1, message: "start" },
        { thread: 2, js_context: 0, message: "start" },
        { thread: 2, js_context: 0, message: "end" },
        { thread: 1000, js_context: 0, event_loop: true, message: "start" },
        { thread: 1000, js_context: 0, event_loop: true, message: "end" },
        { thread: 2, js_context: 0, message: "resolved" },
        { thread: 1, message: "end" },
    ]);
});

Deno.test("should_wait_for_code_to_finish_context", async () => {
    const example = "should_wait_for_code_to_finish_context";
    const results = await executeBasicJoin(example);
    assertArraysMatch(results, [
        { thread: 1, message: "start" },
        { thread: 2, js_context: 0, message: "start" },
        { thread: 2, js_context: 0, message: "end" },
        { thread: 1000, js_context: 0, event_loop: true, message: "start" },
        { thread: 1000, js_context: 0, event_loop: true, message: "end" },
        { thread: 2, js_context: 0, message: "resolved" },
        { thread: 1, message: "end" },
    ]);
});

Deno.test("should_wait_for_code_to_finish_context_blocking", async () => {
    const example = "should_wait_for_code_to_finish_context_blocking";
    const results = await executeBasicJoin(example);
    assertArraysMatch(results, [
        { thread: 1, message: "start" },
        { thread: 2, js_context: 0, message: "start" },
        { thread: 2, js_context: 0, message: "end" },
        { thread: 1000, js_context: 0, event_loop: true, message: "start" },
        { thread: 1000, js_context: 0, event_loop: true, message: "end" },
        { thread: 2, js_context: 0, message: "resolved" },
        { thread: 1, message: "end" },
    ]);
});
