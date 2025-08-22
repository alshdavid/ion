const message = "Hello World"

globalThis.exports = function handler(req, res) {
  // res.write("Hello World1");
  res.writeHead(200);
  // res.write(Uint8Array.from(message.split('').map(letter => letter.charCodeAt(0))));
  res.write("Hello World");
}
