import { Paths } from './paths.ts'

const binName = Deno.build.os === "windows" ? "ion_examples.exe" : "ion_examples"

export async function executeExample(testName: string, args: string[] = [], env: Record<string, string> = {}): Promise<string> {
    const command = new Deno.Command(Paths["~/"]("target", "debug", binName), {
        args: [testName, ...args],
        stdout: "piped",
        stderr: "piped",
        cwd: Paths["~"],
        env: {
            ...Deno.env.toObject(),
            ...env
        }
    });

    const { code, stdout, stderr } = await command.output();

    if (code !== 0) {
        const errorText = new TextDecoder().decode(stderr);
        throw new Error(
            `Test '${testName}' failed with exit code ${code}:\n${errorText}`
        );
    }

    return new TextDecoder().decode(stdout).trim();
}

export interface ExecuteExampleResult {
    stdout: ReadableStream<string>;
    done: Promise<void>;
    pid: number;
    getMemoryUsage(): Promise<number>
}

export function executeExampleStream(
    testName: string,
    args: string[] = [],
    env: Record<string, string> = {}
): ExecuteExampleResult {
    const command = new Deno.Command(Paths["~/"]("target", "debug", binName), {
        args: [testName, ...args],
        stdout: "piped",
        stderr: "inherit",
        cwd: Paths["~"],
        env: {
            ...Deno.env.toObject(),
            ...env
        }
    });

    const child = command.spawn();

    const done = child.status.then(status => {
        if (status.code !== 0) {
            throw new Error(
                `Test '${testName}' failed with exit code ${status.code}`
            );
        }
    });

    let buffer = "";
    const splitLinesStream = new TransformStream<string, string>({
        transform(chunk, controller) {
            buffer = buffer + chunk;
            const lines = buffer.split("\n");

            // Keep the last incomplete line in the buffer
            buffer = lines.pop() || "";

            // Enqueue all complete lines (trimmed)
            for (const line of lines) {
                const trimmed = line.trim();
                if (trimmed) {
                    controller.enqueue(trimmed);
                }
            }
        },
        flush(controller) {
            // Enqueue any remaining buffered content
            if (buffer) {
                const trimmed = buffer.trim();
                if (trimmed) {
                    controller.enqueue(trimmed);
                }
            }
        }
    });

    return {
        stdout: child.stdout
            .pipeThrough(new TextDecoderStream())
            .pipeThrough(splitLinesStream),
        done,
        pid: child.pid,
        getMemoryUsage: async (): Promise<number> => {
            if (Deno.build.os === "windows") {
                const command = new Deno.Command("tasklist", {
                    args: ["/FI", `PID eq ${child.pid}`, "/FO", "CSV", "/NH"],
                    stdout: "piped",
                });
                const { stdout } = await command.output();
                const output = new TextDecoder().decode(stdout);
                const match = output.match(/"([0-9,]+) K"/);
                if (match) {
                    return parseInt(match[1].replace(/,/g, '')) / 1024;
                }
            } else {
                const command = new Deno.Command("ps", {
                    args: ["-o", "rss=", "-p", child.pid.toString()],
                    stdout: "piped",
                });
                const { stdout } = await command.output();
                const output = new TextDecoder().decode(stdout).trim();
                return parseInt(output) / 1024;
            }
            throw new Error("Could not get process memory");
        }
    };
}