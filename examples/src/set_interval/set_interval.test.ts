import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("set_interval", async () => {
    const result = await executeExample("set_interval");
    assertEquals(
        result,
        [
            `"0 Interval Ran"`,
            `"1 Interval Ran"`,
            `"2 Interval Ran"`,
            `"3 Interval Ran"`,
            `"setInterval cleared"`,
        ].join("\n"),
    );
});
