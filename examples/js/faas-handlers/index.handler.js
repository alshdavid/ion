/** @type {import("./faas").HandlerFunc} */
globalThis.handler = async (req, res) => {
    console.log("Handler running")

    // TODO headers
    // res.headers().set("Content-Type", "text/html; charset=utf-8");
    
    res.writeHead(201);
    
    // // TODO TextEncoder
    // const message = new Uint8Array([
    //     // "hello world"
    //     104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
    // ]);
    
    res.write("hello");
    res.write(" world");
    res.end()
};
