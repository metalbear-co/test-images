from flask import Flask, request

import os, signal

try:
    HOST = os.environ['HOST']
except KeyError:
    HOST = "0.0.0.0"

try:
    PORT = os.environ['PORT']
except KeyError:
    PORT = 80

print(f"Will listen on {HOST}:{PORT}")

app = Flask(__name__)


@app.route("/", methods=["GET"])
def get():
    return "OK - GET: Request completed\n"


@app.route("/", methods=["POST"])
def post():
    return "OK - POST: Request completed\n"


@app.route("/", methods=["PUT"])
def put():
    return "OK - PUT: Request completed\n"


@app.route("/", methods=["DELETE"])
def delete():
    return "OK - DELETE: Request completed\n"

def signal_handler(signum, frame):
    signame = signal.Signals(signum).name
    print(f"{signame} ({signum}) received, shutting down the server")

    func = request.environ.get('werkzeug.server.shutdown')
    if func is None:
        raise RuntimeError('Not running with the Werkzeug Server')
    func()

if __name__ == "__main__":
    signal.signal(signal.SIGTERM, signal_handler)
    app.run(host=HOST, port=PORT)
