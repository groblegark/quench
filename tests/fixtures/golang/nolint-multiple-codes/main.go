package main

func main() {
	//nolint:errcheck,gosec // reason: both errors safely ignored
	riskyMultiple()
}

func riskyMultiple() error { return nil }
