package main

import (
	"errors"
	"fmt"
	"regexp"
	"strings"

	"github.com/thedevsaddam/gojsonq"
)

var (
	errInvalidRule              = func(rule string) error { return fmt.Errorf("invalid rule: %s", rule) }
	errInvalidRulePartForRegexp = func(rulePart string) error { return fmt.Errorf("invalid rule part: %s", rulePart) }
	errKeyNotFound              = func(key string) error { return fmt.Errorf("key not found: %s", key) }
	errExcMarkShouldBeFirst     = errors.New("exclamation mark should be first at the rule")
	errNotMatched               = errors.New("not matched")
	errIncorrectTypeForRule     = errors.New("incorrect type for rule")
)

// Rule implements parse and exec loghell rules.
type Rule struct {
	atSignRe   *regexp.Regexp
	excMarkRe  *regexp.Regexp
	excMarkKey string
}

// NewRule returns new rule.
func NewRule(ruleAsAString string) (*Rule, error) {
	// looking for exclamation mark in rule string.
	excMarkIndex := strings.Index(ruleAsAString, "!")
	// looking for atSign mark in rule string.
	atSignMarkIndex := strings.Index(ruleAsAString, "@")

	// rule should contain exclamation mark and at sign mark.
	if excMarkIndex == -1 || atSignMarkIndex == -1 {
		return nil, errInvalidRule(ruleAsAString)
	}

	// excMark should be greater then atSign and atSign is found.
	if excMarkIndex > atSignMarkIndex && atSignMarkIndex != -1 {
		return nil, errExcMarkShouldBeFirst
	}

	excMarkRule := ruleAsAString[excMarkIndex+1 : atSignMarkIndex]
	atSignMarkRule := ruleAsAString[atSignMarkIndex+1:]

	excMarkRuleSplit := strings.Split(excMarkRule, "=")

	if len(excMarkRuleSplit) != 2 {
		return nil, errInvalidRule(excMarkRule)
	}

	excMarkRuleLeft, excMarkRuleRight := excMarkRuleSplit[0], excMarkRuleSplit[1]

	rule := &Rule{
		excMarkKey: excMarkRuleLeft,
	}
	var err error

	rule.excMarkRe, err = regexp.Compile(excMarkRuleRight)
	if err != nil {
		return nil, errInvalidRulePartForRegexp(excMarkRuleRight)
	}
	rule.atSignRe, err = regexp.Compile(atSignMarkRule)
	if err != nil {
		return nil, errInvalidRulePartForRegexp(excMarkRuleRight)
	}

	return rule, nil
}

func (r *Rule) parsePart(ruleAsAString string, start, end int) (*regexp.Regexp, error) {
	rulePart := ruleAsAString[start:end]

	re, err := regexp.Compile(rulePart)
	if err != nil {
		return nil, errInvalidRulePartForRegexp(rulePart)
	}

	return re, nil
}

// Exec exec parsed rule.
func (r *Rule) Exec(logAsString string) (string, error) {
	excMarkRes := gojsonq.New().JSONString(logAsString).Find(r.excMarkKey)
	if excMarkRes == nil {
		return "", errKeyNotFound(r.excMarkKey)
	}

	switch excMarkRes.(type) {
	case string:
	default:
		return "", errIncorrectTypeForRule
	}

	if !r.excMarkRe.MatchString(excMarkRes.(string)) || !r.atSignRe.MatchString(logAsString) {
		return "", errNotMatched
	}

	s := r.atSignRe.FindString(logAsString)
	s = fmt.Sprintf("<span class=\"highlighted\">%s</span>", s)
	logAsString = r.atSignRe.ReplaceAllString(logAsString, s)

	return logAsString, nil
}
