# The animatic's pictures: for each shot of infinite-spell-game-animatic.cut.toml, its card's picture (the shot's first
# `shows` entry) with the slow camera move card-moves.toml gives it, drawn to the shot's voiced length (the render's
# report.json), 1920x1080 at 30 fps, into the shot's recording file. `app video make` runs it from the project root
# (the game repo) whenever a picture, a move or a line changed.
#
#   uv run --with pillow python talks/animatic/make-card-moves.py            the moves (what `app video make` runs)
#   uv run --with pillow python talks/animatic/make-card-moves.py --render   the one command: points each shot at
#       its card's newest still (storyboard/sketches/card-NN-*.png) where one exists, runs `app video make`, and copies
#       the video and subtitles to talks/animatic/infinite-spell-game-animatic-<seconds>s.mp4 / .srt
import argparse
import json
import re
import shutil
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
CUT = HERE / "infinite-spell-game-animatic.cut.toml"
MOVES = HERE / "card-moves.toml"
ROOT = HERE.parent.parent  # the game repo (the project)
# the 15-card day-in-the-life path the animatic follows (storyboard/path.toml is the reel now)
PATH = ROOT / "docs/storyboard/history/path-15-card-day-in-the-life.toml"
CARDS = ROOT / "storyboard"
SKETCHES = CARDS / "sketches"
APP = Path("/Users/paul/coding/atelico/atelico-app-engine/target/debug/app")
W, H, FPS = 1920, 1080, 30
TAIL = 0.2  # seconds past the shot's end, so the render never holds a last frame


@dataclass(frozen=True)
class Move:
    kind: str  # push | pull | pan | clip
    zoom: float = 1.0
    start: tuple[float, float] = (0.5, 0.5)
    end: tuple[float, float] = (0.5, 0.5)


@dataclass(frozen=True)
class Shot:
    id: str
    vo: str
    file: str  # the recording this script makes, relative to the project
    picture: str  # the card's picture, relative to the project


@dataclass(frozen=True)
class Cut:
    name: str
    shots: list[Shot]


def read_cut() -> Cut:
    raw = tomllib.loads(CUT.read_text())
    shots = []
    for sid in raw["order"]:
        s = raw["shot"][sid]
        rec = s["source"]["recording"]
        shots.append(Shot(id=sid, vo=s["vo"], file=rec["file"], picture=rec["shows"][0]))
    return Cut(name=raw["name"], shots=shots)


def read_moves() -> dict[str, Move]:
    raw = tomllib.loads(MOVES.read_text())["move"]
    out = {}
    for sid, m in raw.items():
        kind = m["kind"]
        assert kind in ("push", "pull", "pan", "clip"), f"{sid}: unknown move {kind}"
        out[sid] = Move(kind=kind, zoom=float(m.get("zoom", 1.0)), start=tuple(m.get("from", (0.5, 0.5))), end=tuple(m.get("to", (0.5, 0.5))))
    return out


def card_lines() -> list[tuple[str, str]]:
    """(card id, spoken line) in play order."""
    cards = tomllib.loads(PATH.read_text())["cards"]
    return [(c, tomllib.loads((CARDS / f"{c}.toml").read_text())["spoken"]) for c in cards]


def check_lines(cut: Cut) -> None:
    """The cut speaks every card's line word for word, in path order."""
    cards = card_lines()
    assert len(cards) == len(cut.shots), f"{len(cards)} cards, {len(cut.shots)} shots"
    for (card, spoken), shot in zip(cards, cut.shots):
        assert card.split("/")[-1] == shot.id, f"shot {shot.id} is not card {card}"
        assert spoken == shot.vo, f"{shot.id}: the cut's line differs from {card}.toml's spoken"


def seconds_of(cut: Cut, root: Path) -> dict[str, float]:
    report = json.loads((root / ".atelico" / cut.name / "report.json").read_text())
    return {s["id"]: s["seconds"] for s in report["shots"]}


def ease(t: float) -> float:
    """Half smoothstep, half linear: soft ends that never stop (the render's freeze scan fails a picture still for 1.5 s)."""
    t = min(max(t, 0.0), 1.0)
    return 0.5 * t + 0.5 * t * t * (3 - 2 * t)


def view(m: Move, t: float) -> tuple[float, float, float]:
    """(zoom, centre x, centre y) at t in 0..1."""
    k = ease(t)
    if m.kind == "push":
        return 1 + (m.zoom - 1) * k, 0.5 + (m.end[0] - 0.5) * k, 0.5 + (m.end[1] - 0.5) * k
    if m.kind == "pull":
        return m.zoom + (1 - m.zoom) * k, m.start[0] + (0.5 - m.start[0]) * k, m.start[1] + (0.5 - m.start[1]) * k
    return m.zoom, m.start[0] + (m.end[0] - m.start[0]) * k, m.start[1] + (m.end[1] - m.start[1]) * k


def encoder(out: Path) -> subprocess.Popen:
    args = ["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-",
            "-c:v", "libx264", "-preset", "medium", "-crf", "16", "-pix_fmt", "yuv420p", str(out)]
    return subprocess.Popen(args, stdin=subprocess.PIPE)


def still_move(picture: Path, m: Move, seconds: float, out: Path) -> None:
    im = Image.open(picture).convert("RGB")
    # fill 16:9 first (a picture of another shape is cropped round its centre)
    pw, ph = im.size
    if abs(pw / ph - W / H) > 0.01:
        cw, ch = (pw, round(pw * H / W)) if pw / ph < W / H else (round(ph * W / H), ph)
        im = im.crop(((pw - cw) // 2, (ph - ch) // 2, (pw - cw) // 2 + cw, (ph - ch) // 2 + ch))
        pw, ph = im.size
    n = round(seconds * FPS)
    enc = encoder(out)
    for f in range(n):
        z, cx, cy = view(m, f / max(n - 1, 1))
        vw, vh = pw / z, ph / z
        x0 = min(max(cx * pw - vw / 2, 0), pw - vw)
        y0 = min(max(cy * ph - vh / 2, 0), ph - vh)
        # sub-pixel crop (no jitter): an affine map from the output to the picture
        frame = im.transform((W, H), Image.Transform.AFFINE, (vw / W, 0, x0, 0, vh / H, y0), resample=Image.Resampling.BICUBIC)
        enc.stdin.write(frame.tobytes())
    enc.stdin.close()
    assert enc.wait() == 0, f"ffmpeg failed on {out}"


def clip_move(clip: Path, seconds: float, out: Path) -> None:
    """A video retimed so its whole move plays over the shot."""
    probe = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(clip)], capture_output=True, text=True, check=True)
    factor = seconds / float(probe.stdout.strip())
    vf = f"setpts={factor:.6f}*PTS,fps={FPS},scale={W}:{H}:flags=lanczos"
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", str(clip), "-an", "-vf", vf, "-t", f"{seconds:.3f}", "-c:v", "libx264", "-preset", "medium", "-crf", "16", "-pix_fmt", "yuv420p", str(out)], check=True)


def make(root: Path) -> None:
    cut = read_cut()
    check_lines(cut)
    moves = read_moves()
    secs = seconds_of(cut, root)
    for s in cut.shots:
        m = moves[s.id]
        out = root / s.file
        out.parent.mkdir(parents=True, exist_ok=True)
        length = secs[s.id] + TAIL
        if m.kind == "clip":
            clip_move(root / s.picture, length, out)
        else:
            still_move(root / s.picture, m, length, out)
        print(f"{s.id}: {m.kind} over {Path(s.picture).name}, {length:.2f} s -> {s.file}")


def newest_still(n: int) -> Path | None:
    found = sorted(SKETCHES.glob(f"card-{n:02d}-*.png"), key=lambda p: p.stat().st_mtime)
    return found[-1] if found else None


def use_new_stills(root: Path) -> None:
    """Points each shot whose card has a card-NN still at its newest one (its picture and its `screen`)."""
    text = CUT.read_text()
    for n, s in enumerate(read_cut().shots, 1):
        still = newest_still(n)
        if still is None or read_moves()[s.id].kind == "clip":
            continue
        rel = still.relative_to(root).as_posix()
        block = re.compile(rf"(\[shot\.{re.escape(s.id)}\][^\[]*?screen = )\"[^\"]*\"(.*?shows = \[)\"[^\"]*\"", re.S)
        text, k = block.subn(rf'\1"Real engine still: {still.name}."\2"{rel}"', text, count=1)
        assert k == 1, f"{s.id}: its block did not match"
        print(f"{s.id}: {rel}")
    CUT.write_text(text)


def render(root: Path, passthrough: list[str]) -> None:
    use_new_stills(root)
    cut_rel = CUT.relative_to(root).as_posix()
    subprocess.run([str(APP), *passthrough, "video", "make", cut_rel, "--orientation", "landscape"], cwd=root, check=True)
    cut = read_cut()
    made = root / ".atelico" / cut.name
    total = json.loads((made / "report.json").read_text())["total_seconds"]
    stem = HERE / f"{cut.name}-{round(total)}s"
    for old in HERE.glob(f"{cut.name}-*s.*"):
        old.unlink()
    shutil.copyfile(made / "landscape.mp4", stem.with_suffix(".mp4"))
    shutil.copyfile(made / "subtitles.srt", stem.with_suffix(".srt"))
    print(f"{stem.with_suffix('.mp4')} ({total:.1f} s)")


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--render", action="store_true", help="use the newest card stills, render with the engine, copy the video here")
    ap.add_argument("--app", nargs=argparse.REMAINDER, default=[], help="options for `app` before `video` (e.g. --port 7992)")
    a = ap.parse_args()
    project = ROOT
    if a.render:
        render(project, a.app)
    else:
        make(project)
