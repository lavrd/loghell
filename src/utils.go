package main

import (
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func SubLogger(component string) zerolog.Logger {
	return log.With().Str("component", component).Logger()
}
