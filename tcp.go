package main

import (
	"bufio"
	"fmt"
	"net"

	"github.com/rs/zerolog"
)

const (
	TCP = "tcp"
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
		logger: SubLog("tcp"),
	}
}

func (s *TCPServer) Start() error {
	s.logger.Debug().Msgf("starting tcp server on port %d", s.port)

	listener, err := net.Listen(TCP, fmt.Sprintf(":%d", s.port))
	if err != nil {
		return err
	}

	s.listener = listener

	for {
		conn, err := listener.Accept()
		if err != nil {
			return err
		}

		s.logger.Debug().Msgf("new conn %s", conn.RemoteAddr().String())

		go s.Handler(conn)
	}
}

func (s *TCPServer) Handler(conn net.Conn) {
	defer func() {
		s.logger.Debug().Msgf("conn close with %s", conn.RemoteAddr().String())

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

		s.logger.Debug().Msgf("received message from %s | %s", conn.RemoteAddr().String(), scanner.Text())
		s.ws.Send(scanner.Text())
	}
}

func (s *TCPServer) Shutdown() error {
	s.logger.Debug().Msg("shutdown tcp server")
	return s.listener.Close()
}
