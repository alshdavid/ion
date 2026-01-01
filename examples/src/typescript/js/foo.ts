import type { Foo } from "./bar.ts";
import { bar, Bar } from "./bar.ts";

export const foo: Bar & Foo = "foo";
export { bar };
