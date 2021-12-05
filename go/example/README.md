# Example

This app shows how need use loghell. \
App sends logs every tick (random duration between every tick) to loghell.

### Usage

```
Usage of main.go:
  -e string
        set loghell server endpoint (default "127.0.0.1:3031")

# start app
go run main.go -e 127.0.0.1:3031 -t 2.5s
```
