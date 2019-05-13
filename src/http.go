package main

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/rs/zerolog"
)

type HTTPServer struct {
	port   int
	logger zerolog.Logger
	srv    *http.Server
}

func NewHTTPServer(port int) *HTTPServer {
	return &HTTPServer{
		port:   port,
		logger: SubLogger("http"),
	}
}

func (s *HTTPServer) Start() error {
	s.logger.Info().Msgf("starting server on port %d", s.port)

	s.srv = &http.Server{
		Addr:    fmt.Sprintf(":%d", s.port),
		Handler: http.FileServer(http.Dir("./dashboard/")),
	}

	return s.srv.ListenAndServe()
}

func (s *HTTPServer) Shutdown() {
	s.logger.Debug().Msg("shutdown server")

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	if err := s.srv.Shutdown(ctx); err != nil {
		s.logger.Error().Err(err).Msg("shutdown server error")
	}
}
