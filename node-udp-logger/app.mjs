import dgram from 'node:dgram';

const server = dgram.createSocket('udp4');

server.on('error', (err) => {
  console.log(`server error:\n${err.stack}`);
  server.close();
  throw err
});

server.on('message', (msg, rinfo) => {
  console.log(`${rinfo.address}:${rinfo.port}: ${msg}`);
});

server.on('listening', () => {
});

server.bind(31415);

process.on("SIGTERM", () => {
  console.log("SIGTERM signal received, shutting down the server");
  server.close();
});
