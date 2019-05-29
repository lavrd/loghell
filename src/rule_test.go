package main

import (
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	correctCases = []Case{
		{
			"correct #1",
			"!level=error@connection",
			"{\"level\":\"error\", \"message\": \"connection lost\"}",
			"{\"level\":\"error\", \"message\": \"<span class=\"highlighted\">connection</span> lost\"}",
		}, {
			"correct #2",
			"!level=debug@lost",
			"{\"level\":\"debug\", \"message\": \"connection lost\"}",
			"{\"level\":\"debug\", \"message\": \"connection <span class=\"highlighted\">lost</span>\"}",
		}, {
			"correct #3",
			"!message=connection lost@error",
			"{\"level\":\"error\", \"message\": \"connection lost\"}",
			"{\"level\":\"<span class=\"highlighted\">error</span>\", \"message\": \"connection lost\"}",
		},
	}

	incorrectRuleCases = []Case{
		{
			name: "miss exclamation mark",
			rule: "level=debug@error",
		}, {
			name: "miss at sign mark",
			rule: "!level=debug error",
		}, {
			name: "miss =",
			rule: "!level debug@error",
		}, {
			name: "empty rule",
			rule: "",
		},
	}

	incorrectExecCases = []Case{
		{
			name: "exc mark regexp not found",
			rule: "!level=error@connection",
			log:  "{\"level\":\"debug\", \"message\": \"connection lost\"}",
		}, {
			name: "atSign mark regexp not found",
			rule: "!level=error@something",
			log:  "{\"level\":\"error\", \"message\": \"connection lost\"}",
		},
	}
)

type Case struct {
	name   string
	rule   string
	log    string
	output string
}

func TestNewRule(t *testing.T) {
	for _, c := range correctCases {
		t.Run(c.name, func(t *testing.T) {
			rule, err := NewRule(c.rule)
			require.NoError(t, err)
			require.NotNil(t, rule)
		})
	}

	for _, c := range incorrectRuleCases {
		t.Run(c.name, func(t *testing.T) {
			rule, err := NewRule(c.rule)
			require.Error(t, err)
			require.Nil(t, rule)
		})
	}
}

func TestRule_Exec(t *testing.T) {
	for _, c := range correctCases {
		t.Run(c.name, func(t *testing.T) {
			rule, err := NewRule(c.rule)
			require.NoError(t, err)
			require.NotNil(t, rule)

			log, err := rule.Exec(c.log)
			require.NoError(t, err)
			require.Equal(t, c.output, log)
		})
	}

	for _, c := range incorrectExecCases {
		t.Run(c.name, func(t *testing.T) {
			rule, err := NewRule(c.rule)
			require.NoError(t, err)
			require.NotNil(t, rule)

			log, err := rule.Exec(c.log)
			require.Error(t, err)
			require.Empty(t, log)
		})
	}
}
