import { executeExample } from '../../test-utils/run_test.ts'
import { assertEquals } from "jsr:@std/assert@^1"

Deno.test("transformers", async () => {
    const result = await executeExample("transformers")
    assertEquals(result, [
        `{`,
        `  "foo": "bar"`,
        `}`
    ].join('\n'))
});