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
	"strconv"
	"strings"
	"time"

	"github.com/go-shiori/go-epub"
	"github.com/PuerkitoBio/goquery"
	flag "github.com/spf13/pflag"
)

var (
	invalidChars = regexp.MustCompile(`[<>:"/\\|?*\x00-\x1F]`)
	reservedNames = map[string]struct{}{
		"CON": {}, "PRN": {}, "AUX": {}, "NUL": {},
		"COM1": {}, "COM2": {}, "COM3": {}, "COM4": {},
		"COM5": {}, "COM6": {}, "COM7": {}, "COM8": {}, "COM9": {},
		"LPT1": {}, "LPT2": {}, "LPT3": {}, "LPT4": {},
		"LPT5": {}, "LPT6": {}, "LPT7": {}, "LPT8": {}, "LPT9": {},
	}
	arxivURLRe = regexp.MustCompile(`^https://arxiv\.org/abs/(\d{4}\.\d{5})(v\d+)?$`)
)

const (
	ARXIV_HTML_BASE_URL = "https://arxiv.org/html/"
	NUM_ATTEMPTS = 5
)

type ArxivPaper struct {
	Id string
	Html string
}

func main() {
	defer os.RemoveAll("tmp")

	/// Extract CLI arguments
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
	txtFile, err := os.Open(input)
	if err != nil {
		log.Fatal(err)
	}
	defer txtFile.Close()

	scanner := bufio.NewScanner(txtFile)

	for scanner.Scan() {
		line := scanner.Text()
		log.Printf("[+] Started processing %s", line)
		
		paper, err := validateArxivUrl(line)
		if err != nil {
			log.Printf("[!] %s is invalid. Error: %s\n", line, err)
			continue
		}

		err = generateEpub(paper, *output)
		if err != nil {
			log.Printf("[!] Couldn't generate .epub for %s. Error: %s\n", line, err)
			continue
		}
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
	res, err := http.Get(ARXIV_HTML_BASE_URL + id)
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

func generateEpub(paper ArxivPaper, outDir string) error {
	doc, err := goquery.NewDocumentFromReader(strings.NewReader(paper.Html))
	if err != nil {
		return errors.New("couldn't open HTML document")
	}

	// Create .epub
	book, err := epub.NewEpub("My First Book")
	if err != nil {
		log.Fatal(err)
	}

	// Download images found in paper
	log.Printf("[-] Downloading images for %s", paper.Id)
	imgsDir := filepath.Join("tmp", paper.Id)
	err = os.MkdirAll(imgsDir, 0755)
	if err != nil {
		return err
	}
	defer os.RemoveAll(imgsDir)
	
	article := doc.Find("article")
	article.Find("img").Each(func(i int, s *goquery.Selection) {
		src, exists := s.Attr("src")
		if !exists {
			return
		}
		
		err := downloadImage(src, paper.Id, imgsDir)
		if err != nil {
			log.Printf("Couldn't download %s for %s. Error: %s\n", src, paper.Id, err)
		}

		// Make image avilable to .epub builder
		parts := strings.Split(src, "/")
		filename := parts[len(parts)-1]
		imgPath := filepath.Join(imgsDir, filename)
		imgRef, err := book.AddImage(imgPath, "")
		if err != nil {
		    return
		}
		s.SetAttr("src", imgRef)
	})

	titleHtml, err := article.Find("h1.ltx_title").First().Html()
	if err != nil {
	    return err
	}
	_, err = book.AddSection(
		titleHtml,
		"Title",
		"",
		"",
	)

	// Add abstract sections
	abstractCount := 0
	article.Find("div.ltx_abstract").Each(func(i int, s *goquery.Selection) {
		html, err := s.Html()
		if err != nil {
			return
		}
		
		_, err = book.AddSection(
			html,
			"abstract" + strconv.Itoa(abstractCount),
			"",
			"",
		)
		if err != nil {
			return
		}

		abstractCount++
	})

	// Add paper's sections
	sectionCount := 0
	article.Find("section.ltx_section").Each(func(i int, s *goquery.Selection) {
		html, err := s.Html()
		if err != nil {
			return
		}
		
		_, err = book.AddSection(
			html,
			"section" + strconv.Itoa(sectionCount),
			"",
			"",
		)
		if err != nil {
			return
		}

		sectionCount++
	})

	title := article.Find("h1.ltx_title").First().Text()
	cleanFilename := cleanFilename(title) + ".epub"
	epubPath := filepath.Join(outDir, cleanFilename)
	
	err = book.Write(epubPath)
	if err != nil {
		return err
	}

	log.Printf("[~] Generated %s\n", epubPath)
	
	return nil
}

func downloadImage(src string, id string, dir string) error {
	parts := strings.Split(src, "/")
	filename := parts[len(parts)-1]
	client := &http.Client{
		Timeout: 30 * time.Second,
	}
	
	candidates := []string {
		ARXIV_HTML_BASE_URL + src,
		ARXIV_HTML_BASE_URL + id + src,
		ARXIV_HTML_BASE_URL + id + filename,
	}

	for _, url := range candidates {
		for range [NUM_ATTEMPTS]int{} {
			resp, err := client.Get(url)
			if err != nil {
				continue
			}
			defer resp.Body.Close()

			if resp.StatusCode == http.StatusOK {
				path := filepath.Join(dir, filename)
				file, err := os.Create(path)
				if err != nil {
					return err
				}
				defer file.Close()

				_, err = io.Copy(file, resp.Body)
				if err != nil {
					return err
				}

				return nil
			}

			if resp.StatusCode == http.StatusTooManyRequests {
		        time.Sleep(time.Second * 3)
				continue
			}

			break
		}
	}
	
	return errors.New("can't find image source")
}

func cleanFilename(name string) string {
	// Remove invalid characters.
	name = invalidChars.ReplaceAllString(name, "_")

	// Trim spaces and dots from both ends.
	name = strings.Trim(name, " .")

	// Replace whitespace with single underscores.
	name = strings.Join(strings.Fields(name), "_")

	if name == "" {
		return "untitled"
	}

	// Handle reserved Windows device names.
	base := name
	if dot := strings.IndexByte(name, '.'); dot != -1 {
		base = name[:dot]
	}

	if _, reserved := reservedNames[strings.ToUpper(base)]; reserved {
		name = "_" + name
	}

	return name
}