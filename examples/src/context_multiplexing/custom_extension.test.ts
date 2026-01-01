import { executeExample } from '../../test-utils/run_test.ts'
import { assertEquals } from "jsr:@std/assert@^1"

Deno.test("context_multiplexing", async () => {
    const result = await executeExample("context_multiplexing")
    assertEquals(result, [
        `[ctx0] Started`,
        `[ctx1] Started`,
        `[ctx2] Started`,
        `[ctx0]: 2`,
        `[ctx1]: 3`,
        `[ctx1]: 4`,
    ].join('\n'))
});