import * as path from "jsr:@std/path@^1";

const scriptPath = new URL(import.meta.url).pathname;
const testUtilsPath = path.dirname(scriptPath);
const examplesPath = path.dirname(testUtilsPath);
const rootPath = path.dirname(examplesPath);

export const Paths = Object.freeze({
    ["~"]: rootPath,
    ["~/"]: (...segs: string[]) => path.join(rootPath, ...segs),
    ["~/examples"]: examplesPath,
    ["~/examples/"]: (...segs: string[]) => path.join(examplesPath, ...segs),
    ["~/examples/js"]: path.join(examplesPath, "js"),
    ["~/examples/js/"]: (...segs: string[]) =>
        path.join(examplesPath, "js", ...segs),
});
