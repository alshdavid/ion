import { executeExample } from '../../test-utils/run_test.ts'
import { assertEquals } from "jsr:@std/assert@^1"

Deno.test("deferred", async () => {
    const result = await executeExample("deferred")
    assertEquals(result, [
        `Promise Start`,
        `"Eval: Start"`,
        `"Eval: End"`,
        `Promise End`,
        `"Done: 42"`,
    ].join('\n'))
});