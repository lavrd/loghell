package main

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	cases = []struct {
		name          string
		rule          string
		log           string
		expectedBytes []byte
	}{
		{
			"correct",
			"!level=error@connection",
			"{\"level\":\"error\", \"message\": \"connection lost\"}",
			[]byte{27, 91, 49, 109, 27, 91, 51, 49, 109, 102, 97, 116, 97, 108, 27, 91, 48, 109, 27, 91, 48, 109},
		},
	}
)

func TestNewRule(t *testing.T) {
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			rule, err := NewRule(c.rule)
			require.NoError(t, err)
			require.NotNil(t, rule)
		})
	}
}

func TestRule_Exec(t *testing.T) {
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			rule, err := NewRule(c.rule)
			require.NoError(t, err)
			require.NotNil(t, rule)

			log, err := rule.Exec(c.log)
			require.NoError(t, err)
			fmt.Println(log)
			// require.Equal(t, c.expectedBytes, []byte(log[10:32]))
		})
	}
}
