FROM golang:1.13.4-alpine3.10

RUN apk update && apk upgrade && \
    apk add --no-cache git

WORKDIR /loghell
COPY . .

RUN go mod download

EXPOSE 3031
EXPOSE 3032
EXPOSE 3033

ENTRYPOINT ["go", "run", "./src/"]
