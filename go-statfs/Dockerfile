FROM golang:1.23-alpine3.20
WORKDIR /app
# pre-copy/cache go.mod for pre-downloading dependencies and only redownloading them in subsequent builds if they change
COPY go.mod test.txt ./
RUN go mod download && go mod verify

COPY . .
RUN go build -v -o /app/go-statfs ./...

CMD ["/app/go-statfs"]
