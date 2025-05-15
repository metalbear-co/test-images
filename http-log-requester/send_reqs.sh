#!/usr/bin/env bash

_term() { 
  echo "SIGTERM received, exiting" 
  exit 0
}

trap _term SIGTERM

while true
do
  # http-logger will print "hi-from-deployed-app" to stdout.
  curl mirrord-tests-http-logger/log/hi-from-deployed-app
  sleep 0.1
done
