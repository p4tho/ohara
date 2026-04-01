import subprocess
import wave
from common import loading_spinner
from pathlib import Path
from yaspin import inject_spinner
from yaspin.core import Yaspin

def get_wav_duration(filepath: str) -> float:
    """
    Returns duration of a .wav file in seconds
    """
    with wave.open(filepath, 'rb') as wav:
        frames = wav.getnframes()
        rate = wav.getframerate()
        duration = frames / float(rate)
        return duration
        
def seconds_to_ms(seconds: float) -> int:
    return int(seconds * 1000)

@inject_spinner(loading_spinner, "Creating .m4b metadata and ffmpeg instructions...")
def create_m4b_metadata(spinner: Yaspin, wav_dir: Path, output_dir: Path) -> None:
    """
    Creates files.txt and metadata.txt in output_dir to tell ffmpeg
    how to concatenate and add chapters for .m4b
    
    Args:
        wav_dir: Directory path of .wav files
        output_dir: Directory path of files.txt and metadata.txt
    
    Return:
        None
    """
    files_path = output_dir / "files.txt"
    files_lines = []
    metadata_path = output_dir / "metadata.txt"
    metadata_lines = [";FFMETADATA1\n"]
    start_ms = 0
    end_ms = 0
    
    for file in wav_dir.iterdir():
        if file.suffix.lower() == ".wav":
            duration = get_wav_duration(str(file))
            duration_ms = seconds_to_ms(duration)
            end_ms = start_ms + duration_ms
            
            files_lines.append(f"file '{file.resolve().as_posix()}'")
            
            metadata_lines.append(
                "[CHAPTER]\n"
                "TIMEBASE=1/1000\n"
                f"START={start_ms}\n"
                f"END={end_ms}\n"
                f"title={file.stem}\n"
            )
            
            start_ms = end_ms
    
    files_path.write_text("\n".join(files_lines))
    metadata_path.write_text("\n".join(metadata_lines))
    spinner.ok("[*]")
    
@inject_spinner(loading_spinner, "Producing .m4b file...")
def create_m4b(spinner: Yaspin, metadata_dir: Path, output_filepath: Path) -> None:
    """
    Create .m4b file
    
    Args:
        metadata_dir: Location of .m4b metadata in form of files.txt and metadata.files_txt
        output_filepath: Filepath of output .m4b
        
    Return:
        None
    """
    files_txt = metadata_dir / "files.txt"
    metadata_txt = metadata_dir / "metadata.txt"
    if not files_txt.is_file():
        raise FileNotFoundError(f"Missing files list: {files_txt}")
    if not metadata_txt.is_file():
        raise FileNotFoundError(f"Missing chapter metadata: {metadata_txt}")
    
    cmd = [
        "ffmpeg",
        "-y",
        "-f", "concat",
        "-safe", "0",
        "-i", str(files_txt),
        "-f", "ffmetadata",
        "-i", str(metadata_txt),
        "-map_metadata", "1",
        "-map_chapters", "1",
        "-c:a", "aac",
        "-b:a", "64k",
        str(output_filepath),
    ]
    
    try:
        result = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True,
        )
        spinner.ok("[*]")
    except FileNotFoundError:
        spinner.fail("[!]")
        raise RuntimeError("ffmpeg is not installed or not available on PATH.")
    except subprocess.CalledProcessError as e:
        spinner.fail("[!]")
        raise RuntimeError(
            "ffmpeg failed while creating the .m4b file.\n"
            f"Command: {' '.join(cmd)}\n"
            f"stderr:\n{e.stderr}"
        ) from e
