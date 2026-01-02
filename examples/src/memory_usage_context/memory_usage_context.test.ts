import { executeExample } from '../../test-utils/run_test.ts'
import { assert, assertLessOrEqual } from "jsr:@std/assert@^1"

type MemoryUsageReport = {
    value: number,
    change: number,
}

const thresholdMegabytes = 10
const slice = 250

Deno.test("memory_usage_context", async () => {
    const result = await executeExample("memory_usage_context")

    const records: Array<MemoryUsageReport> = result.split('\n').map(v => JSON.parse(v))
    assert(records.length !== 0, "No records")

    const lastRecords = records.slice(slice)

    const firstMemory = lastRecords[0].value
    const lastMemory = lastRecords[lastRecords.length - 1].value

    assertLessOrEqual(lastMemory, firstMemory + thresholdMegabytes)
});
