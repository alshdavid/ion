import {
    assertMemoryLastQuarterGrowth,
    executeMemoryTest,
} from "../../test-utils/memory-leak.ts";

Deno.test("memory_usage_tsfn", async () => {
    const report = await executeMemoryTest("memory_usage_tsfn");
    assertMemoryLastQuarterGrowth(report, 5);
});
