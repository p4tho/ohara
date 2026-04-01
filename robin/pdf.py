import pymupdf4llm
import re
import unicodedata
from common import loading_spinner
from pypdf import PdfReader
from yaspin import inject_spinner
from yaspin.core import Yaspin

def get_pdf_bookmarks(pdf_path: Path) -> list[dict[str, str | int]]:
    """
    Get .pdf bookmarks
    
    Args:
        pdf_path: Filepath of .pdf
        
    Return:
        List of dictionaries {'title', 'page'}
    """
    
    reader = PdfReader(pdf_path)
    bookmarks = []
    stack = list(reversed(reader.outline))
    
    while stack:
        item = stack.pop()

        if isinstance(item, list):
            stack.extend(reversed(item))
            continue

        bookmarks.append({
            "title": item.title,
            "page": reader.get_destination_page_number(item) + 1,  # make it 1-based
        })

    return bookmarks

@inject_spinner(loading_spinner, "Parsing .pdf for section headers and text...")
def get_pdf_sections(spinner: Yaspin, pdf_path: Path) -> list[dict[str, str | int]]:
    """
    Separates .pdf text for each bookmark
    
    Args:
        pdf_path: Filepath of .pdf to parse
    
    Return:
        List of dictionaries {'title', 'page', 'text'}
    """
    raw_text = pymupdf4llm.to_text(
                pdf_path, 
                header=False, 
                footer=False, 
                force_text=False,
                ignore_graphics=True,
                ignore_images=True
                )
    
    # Remove noise from raw text (tables, figures, images, etc.)
    def is_number(s):
        try:
            float(s)  # works for int and decimal
            return True
        except ValueError:
            return False
    filtered_lines = []
    
    for line in raw_text.splitlines():
        stripped = line.strip()
        
        if stripped.startswith(">"): # random listing
            continue
        elif stripped.startswith("Table") or stripped.startswith("Figure"): # table of figure descriptor
            continue
        elif stripped.startswith("==>") and stripped.endswith("<=="): # pictures
            continue
        elif (stripped.startswith("+") and stripped.endswith("+")) or \
             (stripped.startswith("|") and stripped.endswith("|")): # table
            continue
        elif re.match(r"^\[\d+\]", line): # references section
            break
        filtered_lines.append(line)
        
    filtered_text = re.sub(r"\[[^\]]*\]", "", "\n".join(filtered_lines)) # remove reference/citation brackets
    
    # Grab .pdf bookmarks
    section_bookmarks = get_pdf_bookmarks(pdf_path)
    
    if not section_bookmarks: # return entire filtered text as one section
        spinner.write(f"[~] No .pdf bookmarks found!")
        spinner.ok("[*]")
        return [{
            'title': 'whole_text',
            'page': 1,
            'text': filtered_text
        }]
        
    spinner.write(f"[~] Found {len(section_bookmarks)} bookmarks!")
    
    # Retrieve texts for each bookmark
    def heading_candidates(lines, i):
        cur = lines[i].strip()
        nxt = lines[i + 1].strip() if i + 1 < len(lines) else ""
        return [cur, f"{cur} {nxt}".strip()]
    def normalize_heading(s: str) -> str:
        s = unicodedata.normalize("NFKC", s) # unicode normalization
        s = ( # normalize quotes and dashes
            s.replace("’", "'")
             .replace("‘", "'")
             .replace("“", '"')
             .replace("”", '"')
             .replace("–", "-")
             .replace("—", "-")
        )
        s = s.strip().lower()
        s = re.sub(r"\s+", " ", s) # collapse whitespace
        s = re.sub(r"^\d+(\.\d+)*\.?\s+", "", s) # remove leading arabic numbering: "1 ", "1.2 ", "2.3.4 "
        s = re.sub(r"^(?:[ivxlcdm]+)\.?\s+", "", s, flags=re.IGNORECASE) # remove leading roman numerals: "I. ", "II ", "IV. "
        s = re.sub(r"^[a-z]\.\s+", "", s) # remove single-letter prefixes: "A. ", "B ", "E. ", etc.
        s = re.sub(r"\s*-\s*", "-", s) # normalize hyphen spacing
        s = re.sub(r"[:.\-–—\s]+$", "", s) # remove trailing punctuation
        return s
    def validate_title(line: str, title: str) -> bool:
        return normalize_heading(line) == normalize_heading(title)
    
    lines = filtered_text.splitlines()
    section_bookmarks = [x for x in section_bookmarks if x['title'].lower() != 'abstract']
    sections = []
    start_search = 0 # avoid rescanning old lines
    first_header_idx = None
    
    for cur in section_bookmarks: # create own abstract section
        for i in range(len(lines)):
            for candidate in heading_candidates(lines, i):
                if validate_title(candidate, cur['title']):
                    first_header_idx = i
                    break
            if first_header_idx is not None:
                break
        if first_header_idx is not None:
            break

    if first_header_idx is not None:
        abstract_text = "\n".join(lines[:first_header_idx]).strip()
        
        sections.append({
            'title': 'Abstract',
            'page': 1,
            'text': abstract_text
        })
    else:
        spinner.write("[~] Could not determine Abstract (no headers found)")
    
    for idx, cur in enumerate(section_bookmarks):
        cur_idx = None
        next_idx = len(lines)
        
        for i in range(start_search, len(lines)):
            for candidate in heading_candidates(lines, i):
                if validate_title(candidate, cur["title"]):
                    cur_idx = i
                    break
            if cur_idx is not None:
                break
                
        if cur_idx is None:
            spinner.write(f"[~] Can't find {cur['title']} section header")
            continue
        
        if idx + 1 < len(section_bookmarks): # find line index of next section start
            next_title = section_bookmarks[idx + 1]["title"]
    
            for j in range(cur_idx + 1, len(lines)):
                if validate_title(lines[j], next_title):
                    next_idx = j
                    break
        
        section_text = "\n".join(lines[cur_idx:next_idx]).strip()
        
        sections.append({
            'title': cur['title'],
            'page': cur['page'],
            'text': section_text
        })
        
        start_search = cur_idx + 1
    
    spinner.ok("[*]")
    return sections