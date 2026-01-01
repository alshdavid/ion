import { executeExample } from '../../test-utils/run_test.ts'
import { assertEquals } from "jsr:@std/assert@^1"

Deno.test("basic", async () => {
    const result = await executeExample("basic")
    assertEquals(result, "Returned: 2")
});