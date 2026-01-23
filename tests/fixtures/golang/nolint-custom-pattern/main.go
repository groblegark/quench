package main

func main() {
	// OK: Intentionally ignoring error in startup
	//nolint:errcheck
	riskyFunction()
}

func riskyFunction() error { return nil }
