/** @type {import("./binding").HandlerFunc} */
export function handler(req, res) {
    res.writeHead(201);

    const message = new Uint8Array([
        // "hello world"
        104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
    ]);

    res.write("hello");
    res.write(" world");
    res.end();
}
