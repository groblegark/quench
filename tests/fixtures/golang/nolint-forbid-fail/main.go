package main

func main() {
	// OK: This comment exists but govet is forbidden
	//nolint:govet
	riskyFunction()
}

func riskyFunction() {}
