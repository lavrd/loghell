package main

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	cases = []struct {
		name string
		rule string
		log  string
	}{
		{
			"correct",
			"!level@fatal",
			"{\"level\":\"fatal\"}",
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

			// TODO rename eLog (how to name log after exec? after that check ws package for this)
			eLog, err := rule.Exec(c.log)
			require.NoError(t, err)

			shouldContain := []string{"<span style=\"", "\">fatal</span>"}
			for _, sc := range shouldContain {
				require.True(t, strings.Contains(eLog, sc), sc)
			}
		})
	}
}
