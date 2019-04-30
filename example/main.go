package main

import (
	"flag"
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
	_, err := w.conn.Write(p)
	return len(p), err
}

func main() {
	tick := flag.Duration("t", time.Millisecond*500, "set tick duration for send logs to loghell")
	flag.Parse()

	conn, err := net.Dial("tcp", "127.0.0.1:3031")
	if err != nil {
		log.Fatal().Err(err).Msg("cannot connect to loghell")
	}
	defer func() {
		if err := conn.Close(); err != nil {
			log.Error().Err(err).Msg("close connection with loghell error")
		}
	}()

	log.Logger = log.
		Output(zerolog.MultiLevelWriter(
			&LoghellWriter{conn},
			zerolog.ConsoleWriter{Out: os.Stdout},
		)).
		Level(zerolog.DebugLevel)

	for range time.Tick(*tick) {
		log.Debug().Str("component", "example").Msg("example debug log")
	}
}
