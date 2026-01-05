import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("external_value", async () => {
    const result = await executeExample("external_value");
    assertEquals(result, "42");
});
