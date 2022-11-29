#!/usr/bin/env bash

while true
do
  # http-logger will print "hi-from-deployed-app" to stdout.
  curl mirrord-tests-http-logger/log/hi-from-deployed-app
  sleep 0.1
done
