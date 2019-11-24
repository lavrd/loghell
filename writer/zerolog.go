package writer

import "net"

// ZerologWriter implements loghell writer for zerolog.
// It sends every log to the loghell.
type ZerologWriter struct {
	conn net.Conn
}

func (w *ZerologWriter) Write(p []byte) (int, error) {
	_, err := w.conn.Write(p)
	return len(p), err
}

// Close closes connaction with loghell.
func (w *ZerologWriter) Close() error {
	return w.conn.Close()
}

// NewZerolog returns new zerolog writer.
func NewZerolog(loghellEndpoint string) (*ZerologWriter, error) {
	conn, err := net.Dial("tcp", loghellEndpoint)
	return &ZerologWriter{conn: conn}, err
}
