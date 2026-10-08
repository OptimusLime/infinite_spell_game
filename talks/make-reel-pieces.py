# The reel's made piece (talks/reel-twin-moons.cut.toml), into .atelico/reel-pieces/: the last shot, the fireball
# hitting a climber at the seed (a real engine clip), with "Infinite Spell Game" small in the lower right, cut in hard
# on the impact frame (no fade) and held to the end. No black tail: the reel ends on the impact. `app video make`
# runs it from the project root.
#   uv run --with pillow python talks/make-reel-pieces.py
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / ".atelico" / "reel-pieces"
W, H, FPS = 1920, 1080, 30
TITLE_CLIP = ROOT / "storyboard/reel/clips/ink-town-fireball-impact-on-climber-from-flight-1.6s.mp4"
# the clip's first frame of the burst (the fireball meets the climber): the title cuts in on it
IMPACT_FRAME = 11
TITLE = "Infinite Spell Game"
FONT = ROOT / "editor/fonts/Roobert-SemiBold.ttf"
CREAM = (250, 244, 232, 255)


def ffmpeg(*args: str) -> None:
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *args], check=True)


def lower_third(png: Path) -> None:
    """The title small in the lower right: cream type on a soft shadow, no box, no floor."""
    size = 46
    font = ImageFont.truetype(str(FONT), size)
    # no floor (it would cut in with the title and darken the frame at once): a soft dark shadow round the type alone
    layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    x, y = W - 96 - int(ImageDraw.Draw(layer).textlength(TITLE, font=font)), int(H * 0.84)
    shadow = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).text((x + 2, y + 3), TITLE, font=font, fill=(10, 8, 30, 235), stroke_width=6, stroke_fill=(10, 8, 30, 160))
    layer = Image.alpha_composite(layer, shadow.filter(ImageFilter.GaussianBlur(9)))
    ImageDraw.Draw(layer).text((x, y), TITLE, font=font, fill=CREAM)
    layer.save(png)


def title_shot(out: Path) -> None:
    png = OUT / "title-lower-third.png"
    lower_third(png)
    # cut in on the impact frame: off before it, full after it, no fade
    vf = f"[0:v]scale={W}:{H}:flags=lanczos,fps={FPS}[b];[b][1:v]overlay=0:0:shortest=1:enable='gte(n,{IMPACT_FRAME})',format=yuv420p[v]"
    ffmpeg("-i", str(TITLE_CLIP), "-loop", "1", "-framerate", str(FPS), "-i", str(png), "-filter_complex", vf, "-map", "[v]", "-shortest", "-c:v", "libx264", "-crf", "16", str(out))


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    title_shot(OUT / "title-fireball-impact-cut-in-1.6s.mp4")
    print(f"made {OUT}")
