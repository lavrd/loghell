package main

import (
	"context"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"nhooyr.io/websocket"
)

func main() {
	log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: time.RFC3339}).Level(zerolog.ErrorLevel)

	endpoint := flag.String("e", "ws://127.0.0.1:3032/", "set loghell websocket server endpoint")
	rule := flag.String("r", "!level@debug", "set loghell rule")
	flag.Parse()

	u := fmt.Sprintf("%s?rule=%s", *endpoint, *rule)

	log.Logger = log.Logger.With().Str("endpoint", *endpoint).Str("rule", *rule).Logger()

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*15)
	defer cancel()

	c, _, err := websocket.Dial(ctx, u, websocket.DialOptions{})
	if err != nil {
		log.Fatal().Err(err).Msg("dial loghell websocket server error")
	}
	defer func() {
		if err := c.Close(websocket.StatusNormalClosure, "loghell dashboard shutdown"); err != nil {
			log.Error().Err(err).Msg("close connection error")
		}
	}()

	for {
		_, reader, err := c.Reader(context.Background())
		if err != nil {
			log.Error().Err(err).Msg("prepare reader error")
		}

		buff, err := ioutil.ReadAll(reader)
		if err != nil {
			log.Error().Err(err).Msg("read from reader error")
		}

		fmt.Println(string(buff))
	}
}
