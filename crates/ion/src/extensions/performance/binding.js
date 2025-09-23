export default class Performance {
    static now() {
        import.meta.extension.now();
    }
}

export const performance = Performance;

globalThis.performance = performance;
