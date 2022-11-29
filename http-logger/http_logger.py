from flask import Flask
import logging

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

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=80)
