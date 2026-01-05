import { assertEquals } from "jsr:@std/assert@^1";
import { Paths } from "../../test-utils/paths.ts";
import { executeExample } from "../../test-utils/run_test.ts";

Deno.test("custom_resolver", async () => {
    const result = await executeExample("custom_resolver");
    assertEquals(
        result,
        [
            `Custom Resolver Has Run For Path "${Paths["~"]}"`,
            `Custom Resolver Has Run For Path "${Paths["~/examples/"]("js", "modules", "index.js")}"`,
            `Custom Resolver Has Run For Path "${Paths["~/examples/"]("js", "modules", "index.js")}"`,
            `Custom Resolver Has Run For Path "${Paths["~/examples/"]("js", "modules", "a.js")}"`,
            `Custom Resolver Has Run For Path "${Paths["~/examples/"]("js", "modules", "b.js")}"`,
        ].join("\n"),
    );
});
