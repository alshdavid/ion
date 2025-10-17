declare global {
    interface ImportMeta {
        extension: {
            log(...args: Array<string>): void;
            warn(...args: Array<string>): void;
            error(...args: Array<string>): void;
        };
    }
}

export default class Console {
    static log(/** @type {Array<any>} */ ...args: any[]) {
        import.meta.extension.log(...args);
    }

    static error(/** @type {Array<any>} */ ...args: any[]) {
        import.meta.extension.error(...args);
    }

    static warn(/** @type {Array<any>} */ ...args: any[]) {
        import.meta.extension.warn(...args);
    }
}

export const console = Console;
