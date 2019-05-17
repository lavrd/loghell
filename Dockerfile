FROM golang:1.12.5-alpine3.9

RUN apk update && apk upgrade && \
    apk add --no-cache git

WORKDIR /loghell
COPY . .

RUN go mod download

EXPOSE 3031
EXPOSE 3032
EXPOSE 3033

ENTRYPOINT ["go", "run", "./src/"]
