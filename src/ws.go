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

// WSServer implement loghell websocket server.
type WSServer struct {
	conns  map[string]*conn
	srv    *http.Server
	logger zerolog.Logger
}

// NewWSServer returns new websocket server.
func NewWSServer(port int) *WSServer {
	wss := &WSServer{
		conns:  make(map[string]*conn),
		logger: SubLogger("ws"),
	}

	wss.srv = &http.Server{
		Addr:    fmt.Sprintf(":%d", port),
		Handler: wss.Handler(),
	}

	return wss
}

// Handler handle new websocket connections.
func (s *WSServer) Handler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		logger := s.logger.With().Str("addr", r.RemoteAddr).Logger()

		c, err := websocket.Accept(w, r, websocket.AcceptOptions{
			InsecureSkipVerify: true,
		})
		if err != nil {
			logger.Error().Err(err).Msg("accept connection error")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		ruleAsAString := r.URL.Query().Get("rule")
		rule, err := NewRule(ruleAsAString)
		if err != nil {
			if err := c.Close(4001, "was input invalid rule"); err != nil {
				logger.Error().Err(err).Msg("close connection error")
			}
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

// Start start websocket server.
func (s *WSServer) Start() error {
	s.logger.Info().Msgf("starting server on %s", s.srv.Addr)
	return s.srv.ListenAndServe()
}

// Shutdown shutdown websocket server.
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

// PrepareAndSend prepare new log message and send it to websocket connection.
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
