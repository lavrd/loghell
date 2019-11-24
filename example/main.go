package main

import (
	"flag"
	"fmt"
	"math/rand"
	"time"

	"loghell/writer"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func main() {
	rand.Seed(time.Now().UnixNano())

	endpoint := flag.String("e", "127.0.0.1:3031", "set loghell server endpoint")
	flag.Parse()

	writer, err := writer.NewZerolog(*endpoint)
	if err != nil {
		log.Fatal().Err(err).Msg("cannot connect to loghell")
	}

	log.Logger = log.
		Output(writer).
		Level(zerolog.DebugLevel)

	for {
		log.Debug().Str("component", "example").Msg("example debug log")
		dur, _ := time.ParseDuration(fmt.Sprintf("%dms", rand.Intn(4500)+501))
		time.Sleep(dur)
	}
}
