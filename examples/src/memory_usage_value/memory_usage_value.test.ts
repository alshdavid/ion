import {
    assertMemoryLastQuarterGrowth,
    executeMemoryTest,
} from "../../test-utils/memory-leak.ts";

// Looks like there might be a memory leak here
Deno.test.ignore("memory_usage_value", async () => {
    const report = await executeMemoryTest("memory_usage_value");
    assertMemoryLastQuarterGrowth(report, 5);
});
