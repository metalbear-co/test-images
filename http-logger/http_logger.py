from flask import Flask, request

import logging, signal

log = logging.getLogger("werkzeug")
log.disabled = True


app = Flask(__name__)


@app.route("/", methods=["GET"])
def get():
    print("GET: Request completed")
    return "GET"

@app.route("/log/<string:log>", methods=["GET"])
def log_get(log):
    print(log, flush=True)
    return log

def signal_handler(signum, frame):
    signame = signal.Signals(signum).name
    print(f"{signame} ({signum}) received, shutting down the server")

    func = request.environ.get('werkzeug.server.shutdown')
    if func is None:
        raise RuntimeError('Not running with the Werkzeug Server')
    func()

if __name__ == "__main__":
    signal.signal(signal.SIGTERM, signal_handler)
    app.run(host="0.0.0.0", port=80)
