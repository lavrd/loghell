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
			return err
		}

		go s.Handler(conn)
	}
}

func (s *TCPServer) Handler(conn net.Conn) {
	s.logger.Debug().Msgf("new connection %s", conn.RemoteAddr().String())

	defer func() {
		s.logger.Debug().Msgf("connection close with %s", conn.RemoteAddr().String())

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
		s.logger.Debug().Msgf("received message from %s | %s", conn.RemoteAddr().String(), log)

		s.ws.PrepareAndSend(log)
	}
}

func (s *TCPServer) Shutdown() error {
	s.logger.Debug().Msg("shutdown tcp server")
	if err := s.listener.Close(); err != nil {
		s.logger.Error().Err(err).Msgf("shutdown server error")
		return err
	}
	return nil
}
