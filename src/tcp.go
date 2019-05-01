package main

import (
	"bufio"
	"fmt"
	"net"

	"github.com/rs/zerolog"
)

type TCPServer struct {
	port     int
	ws       *WSServer
	listener net.Listener
	logger   zerolog.Logger
}

func NewTCPServer(port int, ws *WSServer) *TCPServer {
	return &TCPServer{
		port:   port,
		ws:     ws,
		logger: SubLogger("tcp"),
	}
}

func (s *TCPServer) Start() error {
	s.logger.Debug().Msgf("starting server on port %d", s.port)

	var err error
	s.listener, err = net.Listen("tcp", fmt.Sprintf(":%d", s.port))
	if err != nil {
		return err
	}

	for {
		conn, err := s.listener.Accept()
		if err != nil {
			s.logger.Error().Err(err).Msg("accept tcp connection error")
			return nil
		}

		go s.Handler(conn, s.logger.With().Str("addr", conn.RemoteAddr().String()).Logger())
	}
}

func (s *TCPServer) Handler(conn net.Conn, logger zerolog.Logger) {
	logger.Debug().Msg("new connection")

	defer func() {
		s.logger.Debug().Msg("connection closed")

		if err := conn.Close(); err != nil {
			s.logger.Error().Err(err)
		}
	}()

	reader := bufio.NewReader(conn)
	scanner := bufio.NewScanner(reader)
	for scanner.Scan() {
		err := scanner.Err()
		if err != nil {
			s.logger.Error().Err(err)
			return
		}

		log := scanner.Text()
		s.logger.Debug().Msgf("received message from | %s", log)
		s.ws.PrepareAndSend(log)
	}
}

func (s *TCPServer) Shutdown() {
	s.logger.Debug().Msg("shutdown tcp server")
	if err := s.listener.Close(); err != nil {
		s.logger.Error().Err(err).Msgf("shutdown server error")
	}
}
