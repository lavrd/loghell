package main

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/rs/zerolog/log"
	"nhooyr.io/websocket"
	"nhooyr.io/websocket/wsjson"
)

const (
	ReasonServerShutdown = "server shutdown"
)

type WSServer struct {
	port  int
	conns map[string]*websocket.Conn
	srv   *http.Server
}

func NewWSServer(port int) *WSServer {
	return &WSServer{
		port:  port,
		conns: make(map[string]*websocket.Conn),
	}
}

func (s *WSServer) Handler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		log.Debug().Msgf("new websocket conn %s", r.RemoteAddr)

		conn, err := websocket.Accept(w, r, websocket.AcceptOptions{})
		if err != nil {
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		s.conns[r.RemoteAddr] = conn
	})
}

func (s *WSServer) Start() error {
	log.Debug().Msgf("starting websocket server on port %d", s.port)

	s.srv = &http.Server{
		Addr:    fmt.Sprintf(":%d", s.port),
		Handler: s.Handler(),
	}

	return s.srv.ListenAndServe()
}

func (s *WSServer) Shutdown() error {
	log.Debug().Msg("shutdown websocket server")

	for _, c := range s.conns {
		if err := c.Close(websocket.StatusNormalClosure, ReasonServerShutdown); err != nil {
			log.Error().Err(err)
		}
	}

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*1)
	defer cancel()

	return s.srv.Shutdown(ctx)
}

func (s *WSServer) Send(v string) {
	for _, c := range s.conns {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second*1)

		if err := wsjson.Write(ctx, c, nil); err != nil {
			log.Error().Err(err)
		}

		cancel()
	}
}
