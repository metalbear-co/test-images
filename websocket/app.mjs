/**
 * Websocket echo server.
 *
 * Replies with "remote {received data}".
 */
import { WebSocketServer } from 'ws';

const wss = new WebSocketServer({ port: 80 });

wss.on('connection', (ws) => {

    ws.on('close', (code, reason) => {
        console.log(`>> ws close with ${code} due to ${reason.toString()}`);
    });

    ws.on('error', (fail) => {
        console.error(`>> ws error in the websocket itself ${fail}`);
    });

    ws.on('message', (data) => {
        console.log('>> ws received: %s', data);

        ws.send(`remote: ${data}`, (fail) => {
            console.error(`>> ws send failed send with ${fail}`);
        });
    });

    ws.on('open', (listener) => {
        console.log('>> ws open websocket');
    });

    ws.on('ping', (data) => {
        console.log(`>> ws ping data ${data.toString()}`);
    });

    ws.on('pong', (data) => {
        console.log(`>> ws pong data ${data.toString()}`);
    });

    ws.on('unexpected-response', (request, reqsponse) => {
        console.error(`>> ws unexpected-response request is ${JSON.stringify(request)} and response is ${JSON.stringify(response)}`);
    });

    ws.on('upgrade', (listener) => {
        console.log('> ws upgrade: we have an upgrade event');
    });
});

wss.on('error', (fail) => {
    console.error(`> wss error in websocket server setup land ${fail}`);
});

wss.on('headers', (headers, request) => {
    console.log(`> wss headers we have some headers ${headers}`);
});

wss.on('listening', () => {
    console.log(`> wss listening is listening`);
});

wss.on('close', () => {
    console.log(`> wss close wss`);
});
