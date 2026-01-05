import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("promise", async () => {
    const result = await executeExample("promise");
    assertEquals(
        result,
        [
            `"[JS] Promise Started"`,
            `Exec Complete (Not Blocked)`,
            `"[JS] Promise Resolved"`,
            `Resolved with: 42`,
        ].join("\n"),
    );
});
