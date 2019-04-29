package main

import (
	"errors"
	"fmt"
	"regexp"
	"strings"
)

var (
	ErrInvalidRule              = func(rule string) error { return fmt.Errorf("invalid rule: %s", rule) }
	ErrInvalidRulePartForRegexp = func(rulePart string) error { return fmt.Errorf("invalid rule part %s", rulePart) }
	ErrExcMarkShouldBeFirst     = errors.New("exclamation mark should be first at the rule")
)

type Rule struct {
	color   string
	atSign  *regexp.Regexp
	excMark *regexp.Regexp
}

func ParseRule(ruleAsAString string) (*Rule, error) {
	excMarkIndex := strings.Index(ruleAsAString, "!")
	atSignMarkIndex := strings.Index(ruleAsAString, "@")

	if excMarkIndex == -1 && atSignMarkIndex == -1 {
		return nil, ErrInvalidRule(ruleAsAString)
	}

	if excMarkIndex > atSignMarkIndex {
		return nil, ErrExcMarkShouldBeFirst
	}

	rule := &Rule{}
	var err error

	if rule.excMark, err = ParsePart(ruleAsAString, excMarkIndex); err != nil {
		return nil, err
	}

	if rule.atSign, err = ParsePart(ruleAsAString, atSignMarkIndex); err != nil {
		return nil, err
	}

	rule.color = RandColor()

	return rule, nil
}

func ParsePart(rule string, index int) (*regexp.Regexp, error) {
	if index != -1 {
		rulePart := rule[:index]

		re, err := regexp.Compile(rulePart)
		if err != nil {
			return nil, ErrInvalidRulePartForRegexp(rulePart)
		}

		return re, nil
	}

	return nil, nil
}

func ParseRules(rules string) ([]*Rule, error) {
	var rsSlice []*Rule

	rs := strings.Split(rules, ",")
	for _, r := range rs {
		rule, err := ParseRule(r)
		if err != nil {
			return nil, err
		}

		rsSlice = append(rsSlice, rule)
	}

	return rsSlice, nil
}
