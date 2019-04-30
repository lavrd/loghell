package main

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/rs/zerolog"
	"nhooyr.io/websocket"
)

type conn struct {
	rule *Rule
	conn *websocket.Conn
}

type WSServer struct {
	port   int
	conns  map[string]*conn
	srv    *http.Server
	logger zerolog.Logger
}

func NewWSServer(port int) *WSServer {
	return &WSServer{
		port:   port,
		conns:  make(map[string]*conn),
		logger: SubLogger("ws"),
	}
}

func (s *WSServer) Handler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ruleAsAString := r.URL.Query().Get("rule")
		rule, err := NewRule(ruleAsAString)
		if err != nil {
			w.WriteHeader(http.StatusBadRequest)
			if _, err := w.Write([]byte(err.Error())); err != nil {
				s.logger.Error().Err(err).Msg("")
			}
			return
		}

		c, err := websocket.Accept(w, r, websocket.AcceptOptions{})
		if err != nil {
			s.logger.Error().Err(err).Msgf("accept connection %s error", r.RemoteAddr)
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		s.logger.Debug().Msgf("new connection %s", r.RemoteAddr)
		s.conns[r.RemoteAddr] = &conn{
			conn: c,
			rule: rule,
		}
	})
}

func (s *WSServer) Start() error {
	s.logger.Debug().Msgf("starting server on port %d", s.port)

	s.srv = &http.Server{
		Addr:    fmt.Sprintf(":%d", s.port),
		Handler: s.Handler(),
	}

	return s.srv.ListenAndServe()
}

func (s *WSServer) Shutdown() error {
	s.logger.Debug().Msg("shutdown server")

	for _, c := range s.conns {
		if err := c.conn.Close(websocket.StatusNormalClosure, "server shutdown"); err != nil {
			s.logger.Error().Err(err)
		}
	}

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*1)
	defer cancel()

	if err := s.srv.Shutdown(ctx); err != nil {
		s.logger.Error().Err(err).Msg("shutdown server error")
		return err
	}
	return nil
}

func (s *WSServer) PrepareAndSend(log string) {
	for addr, c := range s.conns {
		l, err := c.rule.Exec(log)
		if err == nil {
			s.send(c.conn, addr, l)
		}
	}
}

func (s *WSServer) send(conn *websocket.Conn, addr, log string) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second*1)
	defer cancel()

	s.logger.Debug().Msgf("send message to clients | %s", log)

	writer, err := conn.Writer(ctx, websocket.MessageText)
	if err != nil {
		s.logger.Error().Err(err).Msgf("cannot prepare writer for %s", addr)
		delete(s.conns, addr)
		return
	}

	if _, err := writer.Write([]byte(log)); err != nil {
		s.logger.Error().Err(err).Msgf("write message to %s error", addr)
		if err := writer.Close(); err != nil {
			s.logger.Error().Err(err).Msgf("close connection with %s error", addr)
		}
		delete(s.conns, addr)
	}
}
