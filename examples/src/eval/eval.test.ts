import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("eval", async () => {
    const result = await executeExample("eval", ["console.log(42)"]);
    assertEquals(result, "42");
});
