package main

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/rs/zerolog"
	"nhooyr.io/websocket"
	"nhooyr.io/websocket/wsjson"
)

type WSServer struct {
	port   int
	conns  map[string]*websocket.Conn
	srv    *http.Server
	logger zerolog.Logger
}

func NewWSServer(port int) *WSServer {
	return &WSServer{
		port:   port,
		conns:  make(map[string]*websocket.Conn),
		logger: SubLog("ws"),
	}
}

func (s *WSServer) Handler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		s.logger.Debug().Msgf("new websocket connection %s", r.RemoteAddr)

		conn, err := websocket.Accept(w, r, websocket.AcceptOptions{})
		if err != nil {
			s.logger.Error().Err(err).Msgf("accept connection %s error", r.RemoteAddr)
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		s.conns[r.RemoteAddr] = conn
	})
}

func (s *WSServer) Start() error {
	s.logger.Debug().Msgf("starting websocket server on port %d", s.port)

	s.srv = &http.Server{
		Addr:    fmt.Sprintf(":%d", s.port),
		Handler: s.Handler(),
	}

	return s.srv.ListenAndServe()
}

func (s *WSServer) Shutdown() error {
	s.logger.Debug().Msg("shutdown websocket server")

	for _, c := range s.conns {
		if err := c.Close(websocket.StatusNormalClosure, "server shutdown"); err != nil {
			s.logger.Error().Err(err)
		}
	}

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*1)
	defer cancel()

	return s.srv.Shutdown(ctx)
}

func (s *WSServer) Send(v string) {
	s.logger.Debug().Msgf("send message to clients")

	for _, c := range s.conns {
		ctx, cancel := context.WithTimeout(context.Background(), time.Second*1)

		if err := wsjson.Write(ctx, c, nil); err != nil {
			s.logger.Error().Err(err)
		}

		cancel()
	}
}
