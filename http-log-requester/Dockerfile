FROM bash
WORKDIR /test/
COPY send_reqs.sh .
RUN apk --no-cache add curl
ENTRYPOINT ["./send_reqs.sh"]
