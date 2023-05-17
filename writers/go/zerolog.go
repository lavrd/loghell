package writer

import (
	"fmt"
	"net"
)

// ZerologWriter implements loghell writer for zerolog.
// It sends every log to the loghell.
type ZerologWriter struct {
	conn net.Conn
}

func NewZerolog(loghellEndpoint string) (*ZerologWriter, error) {
	conn, err := net.Dial("tcp", loghellEndpoint)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to loghell: %w", err)
	}
	return &ZerologWriter{conn: conn}, nil
}

func (w *ZerologWriter) Write(p []byte) (int, error) {
	_, err := w.conn.Write(p)
	return len(p), err
}

func (w *ZerologWriter) Close() error {
	return w.conn.Close()
}
