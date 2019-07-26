# loghell
Pretty simple log management tool. 
You can connect you app and view only logs that you need.

### Example
![](./dashboard.png)

### [Example app](./example/README.md)

### Docker usage
```bash
# up
docker-compose up -d --build

# down
docker-compose down
```

### Usage
```bash
Usage of loghell:
  -http int
    	set http server port (default 3033)
  -tcp int
    	set tcp server port (default 3031)
  -v	verbose output
  -ws int
    	set ws server port (default 3032)
    	
# start loghell
go run ./src/main.go
```
