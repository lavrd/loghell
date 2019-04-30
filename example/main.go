package main

import (
	"fmt"
	"net"
	"os"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

type LoghellWriter struct {
	conn net.Conn
}

func (w *LoghellWriter) Write(p []byte) (int, error) {
	n, err := w.conn.Write(p)
	if err != nil {
		log.Error().Err(err)
	}

	fmt.Println(n)

	return len(p), nil
}

func main() {
	conn, err := net.Dial("tcp", "127.0.0.1:3031")
	if err != nil {
		panic(err)
		log.Fatal().Err(err)
	}

	log.Logger = log.
		Output(zerolog.MultiLevelWriter(
			&LoghellWriter{conn},
			zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: time.RFC3339},
		)).
		Level(zerolog.DebugLevel)

	for range time.Tick(time.Millisecond * 2500) {
		log.Debug().Str("component", "example app").Msg("example app debug log")
	}
}
