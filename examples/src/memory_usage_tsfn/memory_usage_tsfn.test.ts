import { executeExample } from '../../test-utils/run_test.ts'
import { assert, assertLessOrEqual } from "jsr:@std/assert@^1"

type MemoryUsageReport = {
    value: number,
    change: number,
}

const thresholdMegabytes = 10

Deno.test("memory_usage_tsfn", async () => {
    const result = await executeExample("memory_usage_tsfn")

    const records: Array<MemoryUsageReport> = result.split('\n').map(v => JSON.parse(v))
    assert(records.length !== 0, "No records")

    const lastRecords = records.slice(15)

    const firstMemory = lastRecords[0].value
    const lastMemory = lastRecords[lastRecords.length - 1].value

    assertLessOrEqual(lastMemory, firstMemory + thresholdMegabytes)
});
