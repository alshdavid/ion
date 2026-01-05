import {
    assertMemoryLastQuarterGrowth,
    executeMemoryTest,
} from "../../test-utils/memory-leak.ts";

Deno.test("memory_usage_module", async () => {
    const report = await executeMemoryTest("memory_usage_module");
    assertMemoryLastQuarterGrowth(report, 5);
});
