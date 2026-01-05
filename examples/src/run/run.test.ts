import * as path from "jsr:@std/path@^1";
import { assertEquals } from "jsr:@std/assert@^1";
import { executeExample } from "../../test-utils/run_test.ts";

const scriptPath = new URL(import.meta.url).pathname;
const scriptDir = path.dirname(scriptPath);

Deno.test("run", async () => {
    const result = await executeExample("run", [
        path.join(scriptDir, "fixture.js"),
    ]);
    assertEquals(result, "42");
});
