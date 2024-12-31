from flask import Flask

import os

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


if __name__ == "__main__":
    app.run(host=HOST, port=PORT)
