package main

import (
	"context"
	"fmt"
	"html/template"
	"net/http"
	"time"

	"github.com/rs/zerolog"
)

type TMPLData struct {
	WSPort int
}

type HTTPServer struct {
	logger   zerolog.Logger
	srv      *http.Server
	tmpl     *template.Template
	tmplData *TMPLData
}

func NewHTTPServer(port, wsPort int) (*HTTPServer, error) {

	httpServer := &HTTPServer{
		logger: SubLogger("http"),
		// tmpl:     tmpl,
		tmplData: &TMPLData{WSPort: wsPort},
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/", httpServer.DashboardHandler)
	mux.Handle("/static/", http.StripPrefix("/static/", http.FileServer(http.Dir("./dashboard/static/"))))

	httpServer.srv = &http.Server{
		Addr:    fmt.Sprintf(":%d", port),
		Handler: mux,
	}

	return httpServer, nil
}

func (s *HTTPServer) DashboardHandler(w http.ResponseWriter, r *http.Request) {
	tmpl, err := template.ParseFiles("./dashboard/index.html")
	if err != nil {
		// return nil, err
	}

	if err := tmpl.Execute(w, s.tmplData); err != nil {
		// TODO don't panic pls
		panic(err)
	}
}

func (s *HTTPServer) Start() error {
	s.logger.Info().Msgf("starting server on %s", s.srv.Addr)
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
