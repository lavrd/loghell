package main

import (
	"fmt"
	"io/ioutil"
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
	buff, err := ioutil.ReadAll(conn)
	if err != nil {
		log.Error().Err(err)
		return
	}

	log.Debug().Msgf(string(buff))
}

func (s *TCPServer) Shutdown() error {
	log.Debug().Msg("tcp server shutdown")
	return s.listener.Close()
}
