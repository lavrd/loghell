package main

import (
	"encoding/hex"
	"errors"
	"fmt"
	"regexp"
	"strings"
)

var (
	ErrInvalidRule              = func(rule string) error { return fmt.Errorf("invalid rule: %s", rule) }
	ErrInvalidRulePartForRegexp = func(rulePart string) error { return fmt.Errorf("invalid rule part %s", rulePart) }
	ErrExcMarkShouldBeFirst     = errors.New("exclamation mark should be first at the rule")
	ErrNotMatched               = errors.New("not matched")
)

type Rule struct {
	atSignRe  *regexp.Regexp
	excMarkRe *regexp.Regexp
}

func NewRule(ruleAsAString string) (*Rule, error) {
	excMarkIndex := strings.Index(ruleAsAString, "!")
	atSignMarkIndex := strings.Index(ruleAsAString, "@")

	if excMarkIndex == -1 && atSignMarkIndex == -1 {
		return nil, ErrInvalidRule(ruleAsAString)
	}

	// excMark should be greater then atSign and atSign is found
	if excMarkIndex > atSignMarkIndex && atSignMarkIndex != -1 {
		return nil, ErrExcMarkShouldBeFirst
	}

	rule := &Rule{}
	var err error

	if atSignMarkIndex == -1 {
		rule.excMarkRe, err = rule.parsePart(ruleAsAString, excMarkIndex+1, len(ruleAsAString))
		if err != nil {
			return nil, err
		}
	} else {
		rule.excMarkRe, err = rule.parsePart(ruleAsAString, excMarkIndex+1, atSignMarkIndex)
		if err != nil {
			return nil, err
		}

		rule.atSignRe, err = rule.parsePart(ruleAsAString, atSignMarkIndex+1, len(ruleAsAString))
		if err != nil {
			return nil, err
		}
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
	if !r.excMarkRe.MatchString(log) {
		return "", ErrNotMatched
	}

	if r.atSignRe != nil {
		s := r.atSignRe.FindString(log)
		s = fmt.Sprintf("\x1b[%dm%v\x1b[0m", 31, s)
		fmt.Println(s)
		fmt.Println([]byte(s))
		fmt.Println([]byte("debug"))
		fmt.Println(hex.EncodeToString([]byte(s)))
		fmt.Println(hex.EncodeToString([]byte("debug")))
		fmt.Println("-----------------------------------")
		s = fmt.Sprintf("\x1b[%dm%v\x1b[0m", 1, s)
		log = r.atSignRe.ReplaceAllString(log, s)
	}

	return log, nil
}
