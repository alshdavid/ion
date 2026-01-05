import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("set_timeout", async () => {
    const result = await executeExample("set_timeout");
    assertEquals(result, [`"1"`, `"2"`, `"3"`, `"4"`, `"5"`].join("\n"));
});
