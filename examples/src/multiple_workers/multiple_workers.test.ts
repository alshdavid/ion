import { executeExample } from "../../test-utils/run_test.ts";
import { assertEquals } from "jsr:@std/assert@^1";

Deno.test("multiple_workers", async () => {
    const result = await executeExample("multiple_workers");
    assertEquals(result, [`"wrk1ctx1"`, `"wrk2ctx1"`, `"wrk3ctx1"`].join("\n"));
});
