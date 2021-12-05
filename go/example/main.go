package main

import (
	"flag"
	"fmt"
	"math/rand"
	"time"

	"github.com/lavrxxx/loghell/go/writer"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func main() {
	rand.Seed(time.Now().UnixNano())

	endpoint := flag.String("e", "127.0.0.1:8080", "set loghell server endpoint")
	flag.Parse()

	w, err := writer.NewZerolog(*endpoint)
	if err != nil {
		log.Fatal().Err(err).Msg("cannot connect to loghell")
	}

	log.Logger = log.
		Output(w).
		Level(zerolog.DebugLevel)

	// {"level":"debug","component":"example","time":"2021-12-05T03:47:06+03:00","message":"example debug log"}
	for {
		log.Debug().Str("component", "example").Msg("example debug log")
		dur, _ := time.ParseDuration(fmt.Sprintf("%dms", rand.Intn(4500)+501))
		time.Sleep(dur)
	}
}
