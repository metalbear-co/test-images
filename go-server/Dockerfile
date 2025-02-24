FROM golang:latest AS build

WORKDIR /app

COPY go.mod .
COPY go.sum .

RUN go mod download

COPY main.go .

RUN CGO_ENABLED=0 go build main.go

FROM alpine:latest AS run

COPY --from=build /app /app

WORKDIR /app

CMD ["/app/main"]
