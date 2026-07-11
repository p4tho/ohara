package main

import (
	"fmt"
	"log"
	"os"
	"path/filepath"

	flag "github.com/spf13/pflag"
)

func main() {
	output := flag.StringP("output", "o", "", "output directory")

	flag.Parse()

	if flag.NArg() != 1 {
		log.Fatalf("Usage: %s [flags] <input file>", flag.CommandLine.Name())
	}

	input := flag.Arg(0)

	if err := validateInputFile(input); err != nil {
		log.Fatalf("Invalid input file. Err: %s", err)
	}

	if *output != "" {
		if err := validateOutputDir(*output); err != nil {
			log.Fatal(err)
		}
	}

	fmt.Printf("Input : %s\n", input)
	fmt.Printf("Output: %s\n", *output)
}

func validateInputFile(path string) error {
	// Must exist
	info, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("input file %q does not exist", path)
		}
		return err
	}

	// Must not be a directory
	if info.IsDir() {
		return fmt.Errorf("%q is a directory, not a file", path)
	}

	// Must be a .txt file
	if filepath.Ext(path) != ".txt" {
		return fmt.Errorf("input file must have a .txt extension")
	}

	return nil
}

func validateOutputDir(path string) error {
	dir := filepath.Dir(path)

	info, err := os.Stat(dir)

	// Must exist
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("output directory %q does not exist", dir)
		}
		return err
	}

	// Must be a directory
	if !info.IsDir() {
		return fmt.Errorf("%q is not a directory", dir)
	}

	return nil 
}

