package main

import (
	"bufio"
	"fmt"
	"net"

	"github.com/rs/zerolog/log"
)

const (
	TCP = "tcp"
)

type TCPServer struct {
	port     int
	ws       *WSServer
	listener net.Listener
}

func NewTCPServer(port int, ws *WSServer) *TCPServer {
	return &TCPServer{
		port: port,
		ws:   ws,
	}
}

func (s *TCPServer) Start() error {
	log.Debug().Msgf("starting tcp server on port %d", s.port)

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

		log.Debug().Msgf("new conn %s", conn.RemoteAddr().String())

		go s.Handler(conn)
	}
}

func (s *TCPServer) Handler(conn net.Conn) {
	defer func() {
		log.Debug().Msgf("conn close with %s", conn.RemoteAddr().String())

		if err := conn.Close(); err != nil {
			log.Error().Err(err)
		}
	}()

	reader := bufio.NewReader(conn)
	scanner := bufio.NewScanner(reader)
	for scanner.Scan() {
		err := scanner.Err()
		if err != nil {
			log.Error().Err(err)
			return
		}

		log.Debug().Msgf("message from %s | %s", conn.RemoteAddr().String(), scanner.Text())
	}
}

func (s *TCPServer) Shutdown() error {
	log.Debug().Msg("shutdown tcp server")
	return s.listener.Close()
}
