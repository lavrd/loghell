package main

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	rule1 = "!component@api"
	rule2 = "!level@debug"
	rules = fmt.Sprintf("%s,%s", rule1, rule2)
)

func TestParseRule(t *testing.T) {
	rule, err := ParseRule(rule1)
	require.NoError(t, err)
	require.NotNil(t, rule)
	require.NotNil(t, rule.atSign)
	require.NotNil(t, rule.excMark)

	rule, err = ParseRule(rule2)
	require.NoError(t, err)
	require.NotNil(t, rule)
	require.NotNil(t, rule.atSign)
	require.NotNil(t, rule.excMark)
}

func TestParseRules(t *testing.T) {
	rules, err := ParseRules(rules)
	require.NoError(t, err)

	for _, r := range rules {
		require.NotNil(t, r)
		require.NotNil(t, r.atSign)
		require.NotNil(t, r.excMark)
	}
}
