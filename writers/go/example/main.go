package main

import (
	"flag"
	"fmt"
	"math/rand"
	"os"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"

	"github.com/lavrd/loghell/writers/go/writer"
)

func main() {
	endpoint := flag.String("e", "127.0.0.1:8080", "set loghell server endpoint")
	flag.Parse()

	w, err := writer.NewZerolog(*endpoint)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to initialize loghell writer")
	}

	log.Logger = log.
		Output(zerolog.MultiLevelWriter(
			zerolog.ConsoleWriter{Out: os.Stdout, NoColor: true},
			w,
		)).
		Level(zerolog.DebugLevel)

	// {"level":"debug","component":"example","time":"2021-12-05T03:47:06+03:00","message":"example debug log"}
	for {
		log.Debug().Str("component", "example").Msg("example debug log")
		dur, _ := time.ParseDuration(fmt.Sprintf("%dms", rand.Intn(1000)+500)) //from 500 ms to 1.5s
		time.Sleep(dur)
	}
}
