from flask import Flask

PORT = 80

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
    app.run(host="0.0.0.0", port=PORT)
