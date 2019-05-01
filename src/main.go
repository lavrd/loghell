package main

import (
	"flag"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func main() {
	tcpPort := flag.Int("tcp", 3031, "set tcp server port")
	wsPort := flag.Int("ws", 3032, "set ws server port")
	verbose := flag.Bool("v", false, "verbose output")
	flag.Parse()

	log.Logger = log.
		Output(zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: time.RFC3339}).
		With().
		Caller().
		Logger().
		Level(zerolog.InfoLevel)
	if *verbose {
		log.Logger = log.Level(zerolog.DebugLevel)
	}

	logger := SubLogger("main")
	logger.Info().Msg("starting loghell")

	wsServer := NewWSServer(*wsPort)
	tcpServer := NewTCPServer(*tcpPort, wsServer)

	go func() {
		if err := wsServer.Start(); err != nil {
			logger.Fatal().Err(err).Msg("start websocket server error")
		}
		defer wsServer.Shutdown()
	}()

	go func() {
		if err := tcpServer.Start(); err != nil {
			logger.Fatal().Err(err).Msg("start tcp server error")
		}
		defer tcpServer.Shutdown()
	}()

	interrupt := make(chan os.Signal)
	signal.Notify(interrupt, syscall.SIGKILL, syscall.SIGINT, syscall.SIGQUIT, syscall.SIGTERM)
	<-interrupt
	logger.Debug().Msg("interrupt signal is notified")

	logger.Info().Msg("loghell shutdown")
}
