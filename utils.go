package main

import (
	"fmt"
	"math/rand"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

const MaxIntensity = 255

func RandColor() string {
	return fmt.Sprintf("#%s%s%s",
		getHex(rand.Intn(MaxIntensity)),
		getHex(rand.Intn(MaxIntensity)),
		getHex(rand.Intn(MaxIntensity)),
	)
}

func getHex(num int) string {
	hex := fmt.Sprintf("%x", num)
	if len(hex) == 1 {
		hex = "0" + hex
	}
	return hex
}

func SubLog(component string) zerolog.Logger {
	return log.With().Str("component", component).Logger()
}
