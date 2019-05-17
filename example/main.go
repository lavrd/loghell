package main

import (
	"flag"
	"net"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

// LoghellWriter is a writer for zerolog that sends logs to loghell
type LoghellWriter struct {
	conn net.Conn
}

// Write sends log to loghell
func (w *LoghellWriter) Write(p []byte) (int, error) {
	_, err := w.conn.Write(p)
	return len(p), err
}

func main() {
	tick := flag.Duration("t", time.Millisecond*500, "set tick duration for send logs to loghell")
	endpoint := flag.String("e", "127.0.0.1:3031", "set loghell server endpoint")
	flag.Parse()

	conn, err := net.Dial("tcp", *endpoint)
	if err != nil {
		log.Fatal().Err(err).Msg("cannot connect to loghell")
	}
	defer func() {
		if err := conn.Close(); err != nil {
			log.Error().Err(err).Msg("close connection with loghell error")
		}
	}()

	log.Logger = log.
		Output(&LoghellWriter{conn}).
		Level(zerolog.DebugLevel)

	for range time.Tick(*tick) {
		log.Debug().Str("component", "example").Msg("example debug log")
	}
}
