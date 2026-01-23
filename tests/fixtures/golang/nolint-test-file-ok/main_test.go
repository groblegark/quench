package main

import "testing"

// No comment needed in test files (default: check = "allow")
//nolint:errcheck
func TestSomething(t *testing.T) {
	riskyTestFunction()
}

func riskyTestFunction() error { return nil }
