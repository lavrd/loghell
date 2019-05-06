package main

import (
	"context"
	"flag"
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"strings"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"nhooyr.io/websocket"
)

func main() {
	endpoint := flag.String("e", "ws://127.0.0.1:3032/", "set loghell websocket server endpoint")
	rule := flag.String("r", "!level@debug", "set loghell rule")
	notification := flag.Bool("n", false, "enabled new log notification")
	flag.Parse()

	log.Logger = log.
		Output(zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: time.RFC3339}).
		Level(zerolog.InfoLevel).
		With().
		Str("endpoint", *endpoint).
		Str("rule", *rule).
		Caller().
		Logger()

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*15)
	defer cancel()

	var connAlreadyClosed bool

	u := fmt.Sprintf("%s?rule=%s", *endpoint, *rule)
	c, _, err := websocket.Dial(ctx, u, websocket.DialOptions{})
	if err != nil {
		log.Fatal().Err(err).Msg("dial loghell websocket server error")
	}
	defer func() {
		if connAlreadyClosed {
			return
		}

		if err := c.Close(websocket.StatusNormalClosure, "loghell dashboard shutdown"); err != nil {
			if strings.Contains(err.Error(), io.EOF.Error()) {
				log.Info().Msg("connection closed")
				return
			}
			log.Error().Err(err).Msg("close connection error")
		}
	}()

	format := "%s\n"
	if *notification {
		format = "\a" + format
	}

	for {
		_, reader, err := c.Reader(context.Background())
		if err != nil {
			if strings.Contains(err.Error(), io.EOF.Error()) {
				connAlreadyClosed = true
				log.Info().Msg("connection closed")
				return
			}

			log.Fatal().Err(err).Msg("prepare reader error")
		}

		buff, err := ioutil.ReadAll(reader)
		if err != nil {
			log.Fatal().Err(err).Msg("read from reader error")
		}

		fmt.Printf(format, string(buff))
	}
}
