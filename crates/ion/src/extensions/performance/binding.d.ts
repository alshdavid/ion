export {};

declare global {
    var performance: any;

    interface ImportMeta {
        extension: {
            now(): number;
        };
    }
}
