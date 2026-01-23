package main

func main() {
	// Wrong pattern - should fail because it doesn't start with "// OK:"
	//nolint:errcheck
	riskyFunction()
}

func riskyFunction() error { return nil }
