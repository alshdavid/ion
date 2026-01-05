import {
    assertMemoryLastQuarterGrowth,
    executeMemoryTest,
} from "../../test-utils/memory-leak.ts";

Deno.test("memory_usage_worker", async () => {
    const report = await executeMemoryTest("memory_usage_worker");
    assertMemoryLastQuarterGrowth(report, 5);
});
