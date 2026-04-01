import argparse
import os
import pyttsx3
import shutil
import re
import sys
from audio import create_m4b_metadata, create_m4b
from common import loading_spinner
from pathlib import Path
from pdf import get_pdf_sections
from yaspin import inject_spinner, yaspin
from yaspin.core import Yaspin

tmp_dir = Path("tmp")

def validate_pdf(path: str) -> Path:
    filepath = Path(path)
    if not filepath.is_file():
        raise argparse.ArgumentTypeError(f"File not found: {path}")
    if filepath.suffix.lower() != '.pdf':
        raise argparse.ArgumentTypeError(f"Input must be a .pdf file.")
    return filepath
    
def validate_m4b_output(path: str) -> Path:
    filepath = Path(path)
    if filepath.suffix.lower() != '.m4b':
        raise argparse.ArgumentTypeError("Output must have a .m4b extension.")
    
    parent = filepath.parent
    
    check_dir = parent
    while not check_dir.exists():
        check_dir = check_dir.parent

    if not os.access(check_dir, os.W_OK):
        raise argparse.ArgumentTypeError(f"No write permission for {check_dir}")
        
    return filepath

@inject_spinner(loading_spinner, text="Cleaning tmp/ directory...")
def cleanup(spinner: Yaspin):
    tmp_path = Path("tmp")
    
    if tmp_path.exists() and tmp_path.is_dir():
        try:
            shutil.rmtree(tmp_path)
            spinner.ok("[*]")
        except Exception as e:
            spinner.write(f"[!] Error while deleting directory: {e}")
            spinner.fail("[!]")
    else:
        spinner.ok("[*]")

def safe_filename(s: str) -> str:
    s = s.lower()
    s = re.sub(r"[ \-]+", "_", s) # replace spaces and hyphens with underscore
    s = re.sub(r'[<>:"/\\|?*]', '', s) # remove invalid filesystem characters
    s = re.sub(r'[^a-z0-9_]', '', s) # remove anything not alphanumeric or underscore
    s = re.sub(r'_+', '_', s) # collapse multiple underscores
    s = s.strip('_') # strip leading/trailing underscores
    return s

def main():
    parser = argparse.ArgumentParser(description="Convert a research paper .pdf into a .m4b file.")
    parser.add_argument("pdf_path", type=validate_pdf, help="Filepath of .pdf")
    parser.add_argument("-o", "--output", type=validate_m4b_output, required=True, help="Filepath of .m4b")
    parser.add_argument("-t", "--transcript", action="store_true", help="Create transcript of .m4b")
    args = parser.parse_args()

    try:
        tmp_dir.mkdir(parents=True, exist_ok=True)
        output_filepath = Path(args.output)
        output_filepath.parent.mkdir(parents=True, exist_ok=True)
        wav_dir = tmp_dir / "wav"
        wav_dir.mkdir(parents=True, exist_ok=True)
        sections = get_pdf_sections(args.pdf_path)
        
        # Create individual .wav files to form .m4b metadata
        with yaspin(loading_spinner, text="Converting sections to .wav files... ") as spinner:
            engine = pyttsx3.init()
            
            for i, section in enumerate(sections):
                text = section["text"].strip()
                if not text:
                    continue

                safe_title = safe_filename(section["title"])
                filename = f"{i:02d}_{safe_title}.wav"
                wav_filepath = wav_dir / filename
                
                engine.save_to_file(text, str(wav_filepath))
            
            engine.runAndWait()
            spinner.ok("[*]")
            
        if args.transcript: # create trnascript of .m4b
            transcript_filepath = output_filepath.parent / f"{output_filepath.stem}_transcript.txt"
            
            with open(str(transcript_filepath), "w", encoding="utf-8") as f:
                for section in sections:
                    f.write(f"Title: {section['title']} Page: {section['page']}\n{section['text']}\n")
                    f.write("\n--------------------------\n\n")
            
        # Create .m4b metadata
        create_m4b_metadata(wav_dir=wav_dir, output_dir=tmp_dir)
        
        # Produce .m4b
        create_m4b(metadata_dir=tmp_dir, output_filepath=output_filepath)
        
        return 0
    except Exception as e:
        print(e)
    finally:
        cleanup()

if __name__ == "__main__":
    sys.exit(main())