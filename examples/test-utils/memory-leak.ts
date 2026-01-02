// Credit to ChatGPT

import { executeExampleStream } from './run_test.ts'

type ProcessMessage = {
  value: number,
  change: number,
}

export type MemoryUsageReport = {
  overAllMemoryTrendPerSample: number,
  firstHalfAverage: number,
  secondHalfAverage: number,
  lastQuarterTrendPerSample: number,
  lastQuarterStdDev: number,
  memoryIncreaseFromFirstToSecondHalf: number,
  memoryIncreaseFromFirstToSecondHalfPercent: number,
}

export async function executeMemoryTest(testName: string,
  args: string[] = [],
  env: Record<string, string> = {}): Promise<MemoryUsageReport> {
  const { done, stdout } = await executeExampleStream(testName, args, env)

  const samples: number[] = [];
  const WARMUP_SAMPLES = 5; // Skip initial samples during warmup

  for await (const recordStr of stdout) {
    const record: ProcessMessage = JSON.parse(recordStr + '\n')
    samples.push(record.value);
    // console.log(`Memory: ${record.value}KB (${record.change >= 0 ? '+' : ''}${record.change}KB)`);
  }

  await done

  // Analysis after process completes
  if (samples.length < 15) {
    throw new Error('Not enough samples collected for reliable leak detection (need at least 15)');
  }

  // Skip warmup phase and analyze the rest
  const steadyStateSamples = samples.slice(WARMUP_SAMPLES);

  // Strategy 1: Check if memory trend is consistently upward
  const overallTrend = calculateTrend(steadyStateSamples);
  // console.log(`\nOverall memory trend: ${overallTrend > 0 ? '+' : ''}${overallTrend.toFixed(2)}KB per sample`);

  // Strategy 2: Compare first half vs second half
  const midpoint = Math.floor(steadyStateSamples.length / 2);
  const firstHalf = steadyStateSamples.slice(0, midpoint);
  const secondHalf = steadyStateSamples.slice(midpoint);

  const firstHalfAvg = average(firstHalf);
  const secondHalfAvg = average(secondHalf);
  const halfDiff = secondHalfAvg - firstHalfAvg;

  // console.log(`First half average: ${firstHalfAvg.toFixed(2)}KB`);
  // console.log(`Second half average: ${secondHalfAvg.toFixed(2)}KB`);
  // console.log(`Difference: ${halfDiff >= 0 ? '+' : ''}${halfDiff.toFixed(2)}KB`);

  // Strategy 3: Check for stabilization
  const lastQuarter = steadyStateSamples.slice(-Math.floor(steadyStateSamples.length / 4));
  const lastQuarterTrend = calculateTrend(lastQuarter);
  const lastQuarterStdDev = standardDeviation(lastQuarter);

  // console.log(`Last quarter trend: ${lastQuarterTrend > 0 ? '+' : ''}${lastQuarterTrend.toFixed(2)}KB per sample`);
  // console.log(`Last quarter std dev: ${lastQuarterStdDev.toFixed(2)}KB`);

  // Detect leak based on multiple signals
  // const leakSignals: string[] = [];

  // Signal 1: Strong upward trend overall (>512KB per sample)
  // if (overallTrend > 512) {
  //   leakSignals.push(`Strong upward trend: ${overallTrend.toFixed(2)}KB per sample`);
  // }

  // Signal 2: Second half significantly higher than first (>20MB or >20% increase)
  const percentIncrease = (halfDiff / firstHalfAvg) * 100;
  // if (halfDiff > 20480 || percentIncrease > 20) {
  //   leakSignals.push(`Memory increased ${halfDiff.toFixed(2)}KB (${percentIncrease.toFixed(1)}%) from first to second half`);
  // }

  // Signal 3: Memory still climbing at the end (trend in last quarter >307KB)
  // if (lastQuarterTrend > 307) {
  //   leakSignals.push(`Memory still climbing at end: ${lastQuarterTrend.toFixed(2)}KB per sample in last quarter`);
  // }

  // Signal 4: High variance in last quarter suggests unstable memory (may indicate leak)
  // if (lastQuarterStdDev > 15360 && lastQuarterTrend > 205) {
  //   leakSignals.push(`High memory variance with upward trend: ${lastQuarterStdDev.toFixed(2)}KB std dev`);
  // }

  // if (leakSignals.length >= 2) {
    // throw new Error(
    //   `Memory leak detected (${leakSignals.length} signals):\n` +
    //   leakSignals.map(s => `  - ${s}`).join('\n')
    // );
  // }

  // if (leakSignals.length === 1) {
  //   console.warn(`\n⚠ Potential leak indicator: ${leakSignals[0]}`);
  // }

  // console.log('\n✓ No significant memory leak detected');

  return {
    overAllMemoryTrendPerSample: overallTrend,
    firstHalfAverage: firstHalfAvg,
    secondHalfAverage: secondHalfAvg,
    lastQuarterTrendPerSample: lastQuarterTrend,
    lastQuarterStdDev: lastQuarterStdDev,
    memoryIncreaseFromFirstToSecondHalf: halfDiff,
    memoryIncreaseFromFirstToSecondHalfPercent: percentIncrease,
  }
};

function average(samples: number[]): number {
  return samples.reduce((a, b) => a + b, 0) / samples.length;
}

function standardDeviation(samples: number[]): number {
  const avg = average(samples);
  const squareDiffs = samples.map(value => Math.pow(value - avg, 2));
  return Math.sqrt(average(squareDiffs));
}

function calculateTrend(samples: number[]): number {
  const n = samples.length;
  const indices = Array.from({ length: n }, (_, i) => i);

  const sumX = indices.reduce((a, b) => a + b, 0);
  const sumY = samples.reduce((a, b) => a + b, 0);
  const sumXY = indices.reduce((sum, x, i) => sum + x * samples[i], 0);
  const sumX2 = indices.reduce((sum, x) => sum + x * x, 0);

  const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
  return slope;
}