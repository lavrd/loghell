package main

import (
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

// SubLogger returns logger with component
func SubLogger(component string) zerolog.Logger {
	return log.With().Str("component", component).Logger()
}
