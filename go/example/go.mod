module example

go 1.19

replace github.com/lavrd/loghell/go/writer => ../writer

require (
	github.com/lavrd/loghell/go/writer v0.0.0
	github.com/rs/zerolog v1.28.0
)

require (
	github.com/mattn/go-colorable v0.1.12 // indirect
	github.com/mattn/go-isatty v0.0.14 // indirect
	golang.org/x/sys v0.1.0 // indirect
)
