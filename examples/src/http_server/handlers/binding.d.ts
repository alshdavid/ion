export type HandlerFunc = (req: Request, res: Response) => any | Promise<any>;

export interface Request {
    body: Reader;
}

export interface Response extends Writer, Ender {
    headers(): Headers;
    writeHead(status: number): Promise<void>;
}

export interface Headers {
    set(header: string, value: string): void;
}

export interface Reader {
    // Read populates the given byte slice with data and returns the number of bytes populated and an error value. It returns null when the stream ends.
    read(recv: Array<number>): Promise<number | null>;
    // Read populates the given byte slice with data and returns the number of bytes populated and an error value. It returns null when the stream ends.
    read(recv: ArrayBuffer): Promise<number | null>;
    // Read populates the given byte slice with data and returns the number of bytes populated and an error value. It returns null when the stream ends.
    read(recv: Uint8Array): Promise<number | null>;
}

export interface Writer extends Ender {
    // Write writes bytes from the buffer to the underlying data stream.
    write(bytes: Array<number>): Promise<number>;
    // Write writes bytes from the buffer to the underlying data stream.
    write(bytes: ArrayBuffer): Promise<number>;
    // Write writes bytes from the buffer to the underlying data stream.
    write(bytes: Uint8Array): Promise<number>;
    // Write writes bytes from the buffer to the underlying data stream.
    write(string: string): Promise<number>;
    // Flush flushes buffered data to the client
    flush(): Promise<void>;
}

export interface Ender {
    end(): Promise<void>;
}

declare global {
    var setTimeout: (
        callback: () => any | Promise<any>,
        duration?: number,
    ) => number;
}
