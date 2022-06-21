from os import getcwd, remove, getpid, kill
from signal import SIGTERM
import click
from flask import Flask, request
import logging
import sys

log = logging.getLogger("werkzeug")
log.disabled = True

cli = sys.modules["flask.cli"]

PORT=5432

cli.show_server_banner = lambda *x: click.echo(f"Server listening on port {PORT}")

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
