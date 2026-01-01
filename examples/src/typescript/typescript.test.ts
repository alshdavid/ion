import { executeExample } from '../../test-utils/run_test.ts'
import { assertEquals } from "jsr:@std/assert@^1"

Deno.test("typescript", async () => {
    const result = await executeExample("typescript")
    assertEquals(result, [
        `"foo"`,
        `"bar"`,
    ].join('\n'))
});