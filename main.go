package main

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"regexp"

	flag "github.com/spf13/pflag"
)

var arxivURLRe = regexp.MustCompile(`^https://arxiv\.org/abs/(\d{4}\.\d{5})(v\d+)?$`)
const ARXIV_HTML_BASE = "https://arxiv.org/html/"

type ArxivPaper struct {
	Id string
	Html string
}

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

	/// Iterate through each line in .txt to generate .epubs
	txt_file, err := os.Open(input)
	if err != nil {
		log.Fatal(err)
	}
	defer txt_file.Close()

	scanner := bufio.NewScanner(txt_file)

	for scanner.Scan() {
		line := scanner.Text()
		log.Printf("[+] Started processing %s", line)
		
		paper, err := validateArxivUrl(line)
		if err != nil {
			log.Printf("[!] %s is invalid. Error: %s\n", line, err)
			continue
		}

		fmt.Printf("%s\n", paper.Id)
	}
	
	if err := scanner.Err(); err != nil {
		log.Fatal(err)
	}
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
		return errors.New("input file must have a .txt extension")
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

func validateArxivUrl(url string) (ArxivPaper, error) {
	// Check arxiv URL format
	matches := arxivURLRe.FindStringSubmatch(url)
	if matches == nil {
		return ArxivPaper{}, errors.New("invalid arXiv URL")
	}
	id := matches[1]

	// Grab raw HTML if possible
	res, err := http.Get(ARXIV_HTML_BASE + id)
	if err != nil {
		return ArxivPaper{}, err
	}
	defer res.Body.Close()
	if res.StatusCode != 200 {
		return ArxivPaper{}, errors.New("HTML format doesn't exist")
	}

	body, err := io.ReadAll(res.Body)
    if err != nil {
        return ArxivPaper{}, err
    }
    
    paper := ArxivPaper{
        Id:   id,
        Html: string(body),
    }

	return paper, nil
}
