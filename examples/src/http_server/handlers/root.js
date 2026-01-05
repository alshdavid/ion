/** @type {import("./binding").HandlerFunc} */
export function handler(req, res) {
    res.writeHead(200);

    res.write("hello");
    res.write(" world");
    res.end();
}
