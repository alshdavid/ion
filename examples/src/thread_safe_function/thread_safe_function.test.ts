import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("thread_safe_function", async () => {
    const result = await executeExample("thread_safe_function");
    assertEquals(result, [`Ret: 2`, `Ret: 4`].join("\n"));
});
