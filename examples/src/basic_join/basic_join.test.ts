import { executeExample } from '../../test-utils/run_test.ts'
import { assertEquals } from "jsr:@std/assert@^1"

Deno.test("basic_join", async () => {
    const result = await executeExample("basic_join")
    assertEquals(result, "Returned: 2")
});