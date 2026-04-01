import argparse
import os
import shutil
import sys
from common import loading_spinner
from pathlib import Path
from pdf import get_pdf_sections
from yaspin import inject_spinner
from yaspin.core import Yaspin

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

def main():
    parser = argparse.ArgumentParser(description="Convert a research paper .pdf into a .m4b file.")
    parser.add_argument("pdf_path", type=validate_pdf, help="Filepath of .pdf")
    parser.add_argument("-o", "--output", type=validate_m4b_output, required=True, help="Filepath of .m4b")
    args = parser.parse_args()

    try:
        sections = get_pdf_sections(args.pdf_path)
    except Exception as e:
        print(e)
    finally:
        cleanup()

if __name__ == "__main__":
    sys.exit(main())