import {
    assertMemoryLastQuarterGrowth,
    executeMemoryTest,
} from "../../test-utils/memory-leak.ts";

Deno.test("memory_usage_external_value", async () => {
    const report = await executeMemoryTest("memory_usage_external_value");
    assertMemoryLastQuarterGrowth(report, 5);
});
