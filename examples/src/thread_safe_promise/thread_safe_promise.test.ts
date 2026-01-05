import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("thread_safe_promise", async () => {
    const result = await executeExample("thread_safe_promise");
    assertEquals(
        result,
        [
            `"[JS] Promise Started"`,
            `Exec Complete (Not Blocked)`,
            `"[JS] Promise Resolved"`,
            `[Rust] Got 42`,
        ].join("\n"),
    );
});
