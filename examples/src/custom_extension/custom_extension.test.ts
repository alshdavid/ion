import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("custom_extension", async () => {
    const result = await executeExample("custom_extension");
    assertEquals(result, "Got: bar");
});
