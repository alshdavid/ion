import { executeMemoryTest } from "../../test-utils/memory-leak.ts";
import { assertLessOrEqual } from "jsr:@std/assert@^1";

Deno.test("memory_usage_external_value", async () => {
    const report = await executeMemoryTest("memory_usage_external_value");

    assertLessOrEqual(
        report.overAllMemoryTrendPerSample,
        5,
        `Memory usage in the last quarter increase at an average of ${report.overAllMemoryTrendPerSample.toFixed(
            0
        )}kb per sample`
    );
});
