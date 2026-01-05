/** @type {import("./binding").HandlerFunc} */
export function handler(req, res) {
    res.headers().set("Content-Type", "text/html; charset=utf-8");

    res.writeHead(200);

    res.write("hello");
    res.write(" world");
    res.end();
}
