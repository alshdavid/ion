declare module "ion:console" {
    export default class Console {
        static log(...args: any[]): void
        static error(...args: any[]): void
        static warn(...args: any[]): void
    }

    export const console: typeof Console;
}
