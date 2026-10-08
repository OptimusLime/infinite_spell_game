# The reel's made pieces (talks/reel-twin-moons.cut.toml), into .atelico/reel-pieces/: the title shot (the ink town
# clip with "Infinite Spell Game" small in the lower third, fading in), the black tail, and the editor stand-in (a slow
# push over a real editor still, until the editor-claude clip lands). `app video make` runs it from the project root.
#   uv run --with pillow python talks/make-reel-pieces.py
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / ".atelico" / "reel-pieces"
W, H, FPS = 1920, 1080, 30
TITLE_CLIP = ROOT / "storyboard/reel/clips/ink-town-weather-firefall-low-orange-moon-4s.mp4"
TITLE = "Infinite Spell Game"
FONT = ROOT / "editor/fonts/Roobert-SemiBold.ttf"
CREAM = (250, 244, 232, 255)


def ffmpeg(*args: str) -> None:
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *args], check=True)


def lower_third(png: Path) -> None:
    """The title small in the lower third, left: cream type on a soft shadow over a dark floor, no box."""
    size = 54
    font = ImageFont.truetype(str(FONT), size)
    # a soft dark floor under it, so cream reads on the snow
    layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    floor = ImageDraw.Draw(layer)
    for row in range(int(H * 0.62), H):
        k = (row - H * 0.62) / (H * 0.38)
        floor.line([(0, row), (W, row)], fill=(12, 8, 32, int(150 * k * k)))
    x, y = 96, int(H * 0.80)
    shadow = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).text((x + 2, y + 3), TITLE, font=font, fill=(10, 8, 30, 200))
    layer = Image.alpha_composite(layer, shadow.filter(ImageFilter.GaussianBlur(6)))
    ImageDraw.Draw(layer).text((x, y), TITLE, font=font, fill=CREAM)
    layer.save(png)


def title_shot(out: Path) -> None:
    png = OUT / "title-lower-third.png"
    lower_third(png)
    vf = f"[0:v]scale={W}:{H}:flags=lanczos,fps={FPS}[b];[1:v]format=rgba,fade=t=in:st=0.4:d=0.4:alpha=1[t];[b][t]overlay=0:0,format=yuv420p[v]"
    ffmpeg("-i", str(TITLE_CLIP), "-loop", "1", "-framerate", str(FPS), "-i", str(png), "-filter_complex", vf, "-map", "[v]", "-t", "4", "-c:v", "libx264", "-crf", "16", str(out))


def black_tail(out: Path) -> None:
    ffmpeg("-f", "lavfi", "-i", f"color=c=black:s={W}x{H}:r={FPS}", "-t", "1", "-c:v", "libx264", "-crf", "16", "-pix_fmt", "yuv420p", str(out))


def editor_stand_in(out: Path) -> None:
    sys.path.insert(0, str(ROOT / "talks/animatic"))
    import importlib.util

    spec = importlib.util.spec_from_file_location("moves", ROOT / "talks/animatic/make-card-moves.py")
    moves = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(moves)
    moves.still_move(ROOT / "storyboard/sketches/world/editor-pixel-graph.png", moves.Move(kind="push", zoom=1.6, end=(0.62, 0.4)), 3.2, out)


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    title_shot(OUT / "title-ink-town-lower-third-4s.mp4")
    black_tail(OUT / "black-tail-1s.mp4")
    editor_stand_in(OUT / "editor-pixel-graph-push-3s.mp4")
    print(f"made {OUT}")
