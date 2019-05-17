# example app
This app shows how need use loghell \
App sends logs every tick to loghell

### Usage
```bash
Usage of main.go:
  -e string
    	set loghell server endpoint (default "127.0.0.1:3031")
  -t duration
    	set tick duration for send logs to loghell (default random ms)

# start app
go run main.go -e 127.0.0.1:3031 -t 2.5s
```
