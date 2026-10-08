# The reel's made piece (talks/reel-twin-moons.cut.toml), into .atelico/reel-pieces/: the last shot, the ink town's
# twin moons aligned with fire falling (a real engine clip), with "Infinite Spell Game" small in the lower right (the hero looks up from the lower left),
# sliding up and fading in over 0.35 s. No black tail: the reel ends on the moons. `app video make` runs it from the
# project root.
#   uv run --with pillow python talks/make-reel-pieces.py
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / ".atelico" / "reel-pieces"
W, H, FPS = 1920, 1080, 30
TITLE_CLIP = ROOT / "storyboard/reel/clips/ink-town-twin-moons-aligned-firefall-zoom-3.2s.mp4"
TITLE_SECONDS = 3.2
TITLE = "Infinite Spell Game"
FONT = ROOT / "editor/fonts/Roobert-SemiBold.ttf"
CREAM = (250, 244, 232, 255)


def ffmpeg(*args: str) -> None:
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *args], check=True)


def lower_third(png: Path) -> None:
    """The title small in the lower right: cream type on a soft shadow over a faint dark floor, no box."""
    size = 46
    font = ImageFont.truetype(str(FONT), size)
    # a soft dark floor under it, so cream reads on the snow
    layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    floor = ImageDraw.Draw(layer)
    for row in range(int(H * 0.62), H):
        k = (row - H * 0.62) / (H * 0.38)
        floor.line([(0, row), (W, row)], fill=(12, 8, 32, int(110 * k * k)))
    x, y = W - 96 - int(ImageDraw.Draw(layer).textlength(TITLE, font=font)), int(H * 0.84)
    shadow = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).text((x + 2, y + 3), TITLE, font=font, fill=(10, 8, 30, 200))
    layer = Image.alpha_composite(layer, shadow.filter(ImageFilter.GaussianBlur(6)))
    ImageDraw.Draw(layer).text((x, y), TITLE, font=font, fill=CREAM)
    layer.save(png)


def title_shot(out: Path) -> None:
    png = OUT / "title-lower-third.png"
    lower_third(png)
    # in at 0.5 s: up 18 px and from clear to full over 0.35 s (eased), then held
    rise = "18*pow(1-min(max((t-0.5)/0.35,0),1),2)"
    vf = f"[0:v]scale={W}:{H}:flags=lanczos,fps={FPS}[b];[1:v]format=rgba,fade=t=in:st=0.5:d=0.35:alpha=1[t];[b][t]overlay=x=0:y='{rise}',format=yuv420p[v]"
    ffmpeg("-i", str(TITLE_CLIP), "-loop", "1", "-framerate", str(FPS), "-i", str(png), "-filter_complex", vf, "-map", "[v]", "-t", str(TITLE_SECONDS), "-c:v", "libx264", "-crf", "16", str(out))


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    title_shot(OUT / "title-ink-town-twin-moons-lower-right-3.2s.mp4")
    print(f"made {OUT}")
