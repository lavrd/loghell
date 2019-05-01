package main

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/rs/zerolog"
	"nhooyr.io/websocket"
)

type conn struct {
	rule   *Rule
	conn   *websocket.Conn
	logger zerolog.Logger
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
		logger := s.logger.With().Str("addr", r.RemoteAddr).Logger()

		ruleAsAString := r.URL.Query().Get("rule")
		rule, err := NewRule(ruleAsAString)
		if err != nil {
			w.WriteHeader(http.StatusBadRequest)
			if _, err := w.Write([]byte(err.Error())); err != nil {
				logger.Error().Err(err).Msg("write response error")
			}
			return
		}

		c, err := websocket.Accept(w, r, websocket.AcceptOptions{})
		if err != nil {
			logger.Error().Err(err).Msg("accept connection error")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		go func() {
			_, _, err := c.Reader(context.Background())
			if err != nil {
				if strings.Contains(err.Error(), io.EOF.Error()) {
					logger.Debug().Msg("connection closed")
				} else {
					logger.Error().Err(err).Msg("cannot prepare reader")
				}

				delete(s.conns, r.RemoteAddr)
			}
		}()

		logger.Debug().Msg("new connection")
		s.conns[r.RemoteAddr] = &conn{
			conn:   c,
			rule:   rule,
			logger: logger,
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

func (s *WSServer) Shutdown() {
	s.logger.Debug().Msg("shutdown server")

	for _, c := range s.conns {
		if err := c.conn.Close(websocket.StatusNormalClosure, "server shutdown"); err != nil {
			s.logger.Error().Err(err)
		}
	}

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	if err := s.srv.Shutdown(ctx); err != nil {
		s.logger.Error().Err(err).Msg("shutdown server error")
	}
}

func (s *WSServer) PrepareAndSend(log string) {
	s.logger.Debug().Msgf("send message to clients | %s", log)

	for addr, c := range s.conns {
		l, err := c.rule.Exec(log)
		if err == nil {
			s.send(c, addr, l)
		}
	}
}

func (s *WSServer) send(c *conn, addr, log string) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	writer, err := c.conn.Writer(ctx, websocket.MessageText)
	if err != nil {
		c.logger.Error().Err(err).Msg("cannot prepare writer for")
		return
	}
	defer func() {
		if err := writer.Close(); err != nil {
			c.logger.Error().Err(err).Msg("close connection error")
		}
	}()

	if _, err := writer.Write([]byte(log)); err != nil {
		c.logger.Error().Err(err).Msg("write message error")
	}
}
