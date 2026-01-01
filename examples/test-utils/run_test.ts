export async function executeExample(testName: string): Promise<string> {
    const scriptPath = new URL(import.meta.url).pathname;
    const scriptDir = scriptPath.substring(0, scriptPath.lastIndexOf('/'));

    const command = new Deno.Command("cargo", {
        args: ["run", "-p", "ion_examples", "--", testName],
        stdout: "piped",
        stderr: "piped",
        cwd: scriptDir,
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
