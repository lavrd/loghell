package main

import (
	"flag"
	"math/rand"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func main() {
	rand.Seed(time.Now().UnixNano())

	tcpPort := flag.Int("tcp", 3031, "set tcp server port")
	wsPort := flag.Int("ws", 3032, "set ws server port")
	verbose := flag.Bool("v", false, "verbose output")
	flag.Parse()

	log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: time.RFC3339}).Level(zerolog.ErrorLevel)
	if *verbose {
		log.Logger = log.Level(zerolog.DebugLevel)
	}

	logger := SubLogger("main")

	logger.Debug().Msg("starting loghell")

	wsServer := NewWSServer(*wsPort)
	tcpServer := NewTCPServer(*tcpPort, wsServer)

	go func() {
		if err := wsServer.Start(); err != nil {
			logger.Fatal().Err(err).Msg("start websocket server error")
		}
	}()

	go func() {
		if err := tcpServer.Start(); err != nil {
			logger.Fatal().Err(err).Msg("start tcp server error")
		}
	}()

	interrupt := make(chan os.Signal)
	signal.Notify(interrupt, syscall.SIGKILL, syscall.SIGINT, syscall.SIGQUIT, syscall.SIGTERM)
	<-interrupt
	logger.Debug().Msg("interrupt signal is notified")

	tcpServer.Shutdown()
	wsServer.Shutdown()

	logger.Debug().Msg("loghell shutdown")
}

// todo at first need to prepare log for every ws client and then send to clients
