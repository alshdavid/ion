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
    stderr: ReadableStream<string>;
    done: Promise<void>;
}

export function executeExampleStream(
    testName: string, 
    args: string[] = [], 
    env: Record<string, string> = {}
): ExecuteExampleResult {
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

    const child = command.spawn();

    const done = child.status.then(status => {
        if (status.code !== 0) {
            throw new Error(
                `Test '${testName}' failed with exit code ${status.code}`
            );
        }
    });

    const trimStream = new TransformStream<string, string>({
        transform(chunk, controller) {
            controller.enqueue(chunk.trim());
        }
    });

    return {
        stdout: child.stdout
            .pipeThrough(new TextDecoderStream())
            .pipeThrough(trimStream),
        stderr: child.stderr
            .pipeThrough(new TextDecoderStream()),
        done
    };
}