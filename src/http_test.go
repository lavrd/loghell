package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/require"
)

const (
	httpPort = 2001
	wsPort   = 2002
)

func TestNewHTTPServer(t *testing.T) {
	httpServer, err := NewHTTPServer(httpPort, wsPort)
	require.NoError(t, err)
	require.NotNil(t, httpServer)
}

func TestHTTPServer_DashboardHandler(t *testing.T) {
	httpServer, _ := NewHTTPServer(httpPort, wsPort)
	req, _ := http.NewRequest(http.MethodGet, "/", nil)
	rr := httptest.NewRecorder()
	handler := http.HandlerFunc(httpServer.DashboardHandler)
	handler.ServeHTTP(rr, req)
	require.Equal(t, http.StatusOK, rr.Code)
	require.Equal(t, "text/html; charset=utf-8", rr.Header().Get("content-type"))
}
