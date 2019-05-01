# dashboard
Terminal dashboard for loghell

### Usage
```bash
Usage of dashboard:
  -e string
    	set loghell websocket server endpoint (default "ws://127.0.0.1:3032/")
  -r string
    	set loghell rule (default "!level@debug")
    	
# start dashboard
$ go run main.go -e ws://127.0.0.1:3032/ -r !level@debug
```
