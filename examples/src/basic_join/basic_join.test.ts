import { executeExample } from "../../test-utils/run_test.ts";
import { assert, assertEquals, assertObjectMatch } from "jsr:@std/assert@^1";

type Result = {
    thread: number;
    message: string;
    js_context?: number;
    event_loop?: boolean;
};

async function executeBasicJoin(caseName: string): Promise<Array<Result>> {
    const result = await executeExample("basic_join", [caseName]);
    return result.split("\n").map((record) => JSON.parse(record));
}

type ProcessedRecords = {
    main: Array<Result>;
    jsContexts: Record<number, Array<Result>>;
    eventLoop: Record<number, Array<Result>>;
};

function processResults(input: Array<Result>): ProcessedRecords {
    const processed: ProcessedRecords = {
        main: [],
        jsContexts: {},
        eventLoop: {},
    };

    for (const result of input) {
        // No js_context -> goes to main
        if (result.js_context === undefined) {
            processed.main.push(result);
            continue;
        }

        // Has event_loop flag -> goes to eventLoop
        if (result.event_loop) {
            if (!processed.eventLoop[result.js_context]) {
                processed.eventLoop[result.js_context] = [];
            }
            processed.eventLoop[result.js_context].push(result);
            continue;
        }

        // Has js_context but no event_loop -> goes to jsContexts
        if (!processed.jsContexts[result.js_context]) {
            processed.jsContexts[result.js_context] = [];
        }
        processed.jsContexts[result.js_context].push(result);
    }

    return processed;
}

async function run(caseName: string): Promise<ProcessedRecords> {
    return processResults(await executeBasicJoin(caseName));
}

function assertArraysMatch<T extends Array<unknown>, Y extends Array<unknown>>(
    arr1: T,
    arr2: Y,
    msg?: string
): void {
    return assertObjectMatch({ arr: arr1 }, { arr: arr2 }, msg);
}

Deno.test("should_cancel_when_dropped", async () => {
    const example = "should_cancel_when_dropped";
    const results = await run(example);

    // The code on the main thread will always run
    assertArraysMatch(results.main, [
        { thread: 1, message: "start" },
        { thread: 1, message: "end" },
    ]);

    // The code on the JavaScript thread may or may not run
    assert(
        (results.jsContexts[0] || []).length === 0 ||
            (results.jsContexts[0] || []).length === 2
    );

    // The code on the Event Loop should not run
    assertObjectMatch(results.eventLoop, {});
});

Deno.test("should_cancel_when_dropped_multiple", async () => {
    const example = "should_cancel_when_dropped_multiple";
    const results = await run(example);

    // The code on the main thread will always run
    assertArraysMatch(results.main, [
        { thread: 1, message: "start" },
        { thread: 1, message: "end" },
    ]);

    // The code on the JavaScript thread may or may not run
    assert(
        (results.jsContexts[0] || []).length === 0 ||
            (results.jsContexts[0] || []).length === 4
    );

    // The code on the Event Loop should not run
    assertObjectMatch(results.eventLoop, {});
});

Deno.test("should_cancel_blocking_when_dropped", async () => {
    const example = "should_cancel_blocking_when_dropped";
    const results = await run(example);

    assertArraysMatch(results.main, [
        { thread: 1, message: "start" },
        { thread: 1, message: "end" },
    ]);

    assertObjectMatch(results.jsContexts, {
        "0": [
            { thread: 2, js_context: 0, message: "start" },
            { thread: 2, js_context: 0, message: "end" },
        ],
    });

    // The code on the Event Loop may or may not run, but not progress when the task sleeps
    assertObjectMatch(results.eventLoop, {});
});

Deno.test("should_cancel_blocking_when_dropped_multiple", async () => {
    const example = "should_cancel_blocking_when_dropped_multiple";
    const results = await run(example);

    assertArraysMatch(results.main, [
        { thread: 1, message: "start" },
        { thread: 1, message: "end" },
    ]);

    assertObjectMatch(results.jsContexts, {
        "0": [
            { thread: 2, js_context: 0, message: "start" },
            { thread: 2, js_context: 0, message: "end" },
            { thread: 2, js_context: 0, message: "start" },
            { thread: 2, js_context: 0, message: "end" },
        ],
    });

    // The code on the Event Loop may or may not run, but not progress when the task sleeps
    assertObjectMatch(results.eventLoop, {});
});

Deno.test("should_wait_for_code_to_finish", async () => {
    const example = "should_wait_for_code_to_finish";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_multiple", async () => {
    const example = "should_wait_for_code_to_finish_multiple";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_blocking", async () => {
    const example = "should_wait_for_code_to_finish_blocking";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_worker", async () => {
    const example = "should_wait_for_code_to_finish_worker";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_worker_blocking", async () => {
    const example = "should_wait_for_code_to_finish_worker_blocking";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_context", async () => {
    const example = "should_wait_for_code_to_finish_context";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_context_blocking", async () => {
    const example = "should_wait_for_code_to_finish_context_blocking";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_contexts_blocking", async () => {
    const example = "should_wait_for_code_to_finish_contexts_blocking";
    const results = processResults(await executeBasicJoin(example));
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
            "1": [
                { thread: 2, js_context: 1, message: "start" },
                { thread: 2, js_context: 1, message: "end" },
                { thread: 2, js_context: 1, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
            "1": [
                {
                    thread: 1000,
                    js_context: 1,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 1,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_wait_for_code_to_finish_contexts", async () => {
    const example = "should_wait_for_code_to_finish_contexts";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});

Deno.test("should_not_run_code_after_joining", async () => {
    const example = "should_not_run_code_after_joining";
    const results = await run(example);
    assertObjectMatch(results, {
        main: [
            { thread: 1, message: "start" },
            { thread: 1, message: "did_not_run" },
            { thread: 1, message: "end" },
        ],
        jsContexts: {
            "0": [
                { thread: 2, js_context: 0, message: "start" },
                { thread: 2, js_context: 0, message: "end" },
                { thread: 2, js_context: 0, message: "resolved" },
            ],
        },
        eventLoop: {
            "0": [
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "start",
                },
                {
                    thread: 1000,
                    js_context: 0,
                    event_loop: true,
                    message: "end",
                },
            ],
        },
    });
});
