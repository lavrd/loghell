package main

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestParseRule(t *testing.T) {
	cases := []struct {
		name string
		rule string
		err  error
	}{
		{
			name: "correct rule",
			rule: "!component@api",
			err:  nil,
		},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			rule, err := ParseRule(c.rule)
			require.Equal(t, c.err, err)
			require.NotNil(t, rule)
			t.Logf("%+v\n", rule)
		})
	}
}

func TestExecRule(t *testing.T) {
	rule, err := ParseRule("!level@api")
	require.NoError(t, err)
	require.NotNil(t, rule)

	ExecRule(rule, "{\"level\":\"debug\",\"component\":\"api\",\"time\":\"2019-04-30T09:45:38+03:00\",\"message\":\"hello from api and this is an api component\"}")
}
