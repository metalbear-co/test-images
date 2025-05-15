const net = require('net');
const server = net.createServer();

server.on('connection', handleConnection);
server.listen(80, function () {
  console.log('server listening to %j', server.address());
});
function handleConnection(conn) {
  var remoteAddress = conn.remoteAddress + ':' + conn.remotePort;
  console.log('new client connection from %s', remoteAddress);
  conn.on('data', onConnData);
  conn.once('close', onConnClose);
  conn.on('error', onConnError);

  function onConnData(d) {
    console.log('connection data from %s: %j', remoteAddress, d.toString());
    conn.write('remote: '.concat(d));
  }
  function onConnClose() {
    console.log('connection from %s closed', remoteAddress);
  }
  function onConnError(err) {
    console.log('Connection %s error: %s', remoteAddress, err.message);
  }
}

process.on("SIGTERM", () => {
  console.log("SIGTERM signal received, shutting down the server");
  server.close();
});
