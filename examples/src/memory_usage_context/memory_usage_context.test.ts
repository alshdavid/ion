import {
    assertMemoryLastQuarterGrowth,
    executeMemoryTest,
} from "../../test-utils/memory-leak.ts";

Deno.test("memory_usage_context", async () => {
    const report = await executeMemoryTest("memory_usage_context");
    assertMemoryLastQuarterGrowth(report, 5);
});
