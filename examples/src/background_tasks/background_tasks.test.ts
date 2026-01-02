import { executeExample } from '../../test-utils/run_test.ts'
import { assertEquals } from "jsr:@std/assert@^1"

Deno.test("background_tasks", async () => {
    const result = await executeExample("background_tasks")
    assertEquals(result, [
        `Task [rs]: Started`,
        `Task [js]: Message`,
        `Task [rs]: Ended`,
    ].join('\n'))
});