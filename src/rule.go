package main

import (
	"errors"
	"fmt"
	"regexp"
	"strings"

	"github.com/thedevsaddam/gojsonq"
)

var (
	ErrInvalidRule              = func(rule string) error { return fmt.Errorf("invalid rule: %s", rule) }
	ErrInvalidRulePartForRegexp = func(rulePart string) error { return fmt.Errorf("invalid rule part: %s", rulePart) }
	ErrExcMarkShouldBeFirst     = errors.New("exclamation mark should be first at the rule")
	ErrNotMatched               = errors.New("not matched")
	ErrKeyNotFound              = func(key string) error { return fmt.Errorf("key not found: %s", key) }
)

type Rule struct {
	atSignRe   *regexp.Regexp
	excMarkRe  *regexp.Regexp
	excMarkKey string
}

func NewRule(ruleAsAString string) (*Rule, error) {
	// looking for exclamation mark in rule string
	excMarkIndex := strings.Index(ruleAsAString, "!")
	// looking for atSign mark in rule string
	atSignMarkIndex := strings.Index(ruleAsAString, "@")

	// rule should contain exclamation mark and at sign mark
	if excMarkIndex == -1 || atSignMarkIndex == -1 {
		return nil, ErrInvalidRule(ruleAsAString)
	}

	// excMark should be greater then atSign and atSign is found
	if excMarkIndex > atSignMarkIndex && atSignMarkIndex != -1 {
		return nil, ErrExcMarkShouldBeFirst
	}

	excMarkRule := ruleAsAString[excMarkIndex+1 : atSignMarkIndex]
	atSignMarkRule := ruleAsAString[atSignMarkIndex : len(ruleAsAString)-1]

	excMarkRuleSplit := strings.Split(excMarkRule, "=")

	if len(excMarkRuleSplit) != 2 {
		return nil, ErrInvalidRule(excMarkRule)
	}

	excMarkRuleLeft, excMarkRuleRight := excMarkRuleSplit[0], excMarkRuleSplit[1]

	rule := &Rule{
		excMarkKey: excMarkRuleLeft,
	}
	var err error

	rule.excMarkRe, err = regexp.Compile(excMarkRuleRight)
	if err != nil {
		return nil, ErrInvalidRulePartForRegexp(excMarkRuleRight)
	}
	rule.atSignRe, err = regexp.Compile(atSignMarkRule)
	if err != nil {
		return nil, ErrInvalidRulePartForRegexp(excMarkRuleRight)
	}

	return rule, nil
}

func (r *Rule) parsePart(ruleAsAString string, start, end int) (*regexp.Regexp, error) {
	rulePart := ruleAsAString[start:end]

	re, err := regexp.Compile(rulePart)
	if err != nil {
		return nil, ErrInvalidRulePartForRegexp(rulePart)
	}

	return re, nil
}

func (r *Rule) Exec(log string) (string, error) {
	excMarkRes := gojsonq.New().JSONString(log).Find(r.excMarkKey)
	if excMarkRes == nil {
		return "", ErrKeyNotFound(r.excMarkKey)
	}

	fmt.Println(excMarkRes)

	if !r.excMarkRe.Match(excMarkRes.([]byte)) || !r.atSignRe.MatchString(log) {
		return "", ErrNotMatched
	}

	s := r.atSignRe.FindString(log)
	// 31 - red color
	s = fmt.Sprintf("\x1b[%dm%v\x1b[0m", 31, s)
	// 1 - bold font
	s = fmt.Sprintf("\x1b[%dm%v\x1b[0m", 1, s)
	log = r.atSignRe.ReplaceAllString(log, s)

	return log, nil
}
