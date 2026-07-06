const net = require('net');
const server = net.createServer();

server.on('connection', handleConnection);

// Bind address is configurable via the `HOST` env var (matching the python test image).
// When unset, we let Node pick the default (all interfaces). Setting it to the pod IP lets us
// test the case where an app listens on the pod's external IP instead of loopback.
const host = process.env.HOST;
const onListening = function () {
  console.log('server listening to %j', server.address());
};
if (host) {
  server.listen(80, host, onListening);
} else {
  server.listen(80, onListening);
}
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
