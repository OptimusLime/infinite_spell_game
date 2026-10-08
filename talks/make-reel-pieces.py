# The reel's made pieces (talks/reel-twin-moons.cut.toml), written beside its clips so the cut plays complete from the
# repo (no .atelico/ piece). `app video make` runs it from the project root; it needs ffmpeg and ImageMagick (magick).
#   python3 talks/make-reel-pieces.py
#
# 1. The look shot, storyboard/reel/clips/ink-town-look-graph-rim-light-edit-wipe-2.27s.mp4: the editor's node graph
#    tight on cel-world's Rim Light node, its colour set to the orange moon and its strength stepped 0.35 -> 1.0 -> 1.8
#    -> 2.0 (four captures of a hidden editor on a copy, the node's preview and the big preview turning orange), then
#    the same edit on the town: two offline renders of one moving shot (render-scene, the graph at the two settings),
#    wiped from the cold look to the orange one from the moon's side.
# 2. The ending, storyboard/reel/clips/ink-town-fireball-burst-title-fade-to-black-2.23s.mp4: the fireball's burst on a
#    climber (a real clip) from its impact frame, "Infinite Spell Game" cut in with it in the lower right, the picture
#    fading to black under the title, the title held 1 s on black.
# 3. The music bed, storyboard/reel/audio/reel-twin-moons-bed-rising-moon-from-4.92s-with-hits-27s.wav: "Fantasy:
#    Rising Moon" (RandomMind, CC0; its licence file beside it) from 4.92 s, so its first hit (9.12 s) lands on the
#    reel's second cut and its full section (27.39 s) on the fireball's launch; plus two synthesized hits mixed low: a
#    noise whoosh into the launch and a falling sine thump with a noise burst on the impact frame.
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLIPS = ROOT / "storyboard/reel/clips"
AUDIO = ROOT / "storyboard/reel/audio"
W, H, FPS = 1920, 1080, 30

LOOK_OUT = CLIPS / "ink-town-look-graph-rim-light-edit-wipe-2.27s.mp4"
# the editor's steps (crops of the hidden editor's window: node graph and preview), frames each
EDITOR_STEPS = [("editor-look-graph-rim-lilac-0.35.png", 6), ("editor-look-graph-rim-orange-1.0.png", 6),
                ("editor-look-graph-rim-orange-1.8.png", 6), ("editor-look-graph-rim-orange-2.0.png", 8)]
TOWN_COLD = CLIPS / "ink-town-look-rim-light-lilac-0.35-1.5s.mp4"
TOWN_ORANGE = CLIPS / "ink-town-look-rim-light-orange-2.0-1.5s.mp4"
TOWN_FRAMES, WIPE_AT, WIPE_FRAMES = 42, 10, 14  # the town part; the wipe starts 10 frames in and takes 14

TITLE_OUT = CLIPS / "ink-town-fireball-burst-title-fade-to-black-2.23s.mp4"
TITLE_CLIP = CLIPS / "ink-town-fireball-impact-on-climber-from-flight-1.6s.mp4"
# the clip's first frame of the burst (the fireball meets the climber): the piece starts on it, the title with it;
# the picture fades to black over its last FADE frames, then BLACK frames hold the title on black
IMPACT_FRAME, FADE, BLACK = 11, 15, 30
TITLE = "Infinite Spell Game"
FONT = ROOT / "editor/fonts/Roobert-SemiBold.ttf"

MUSIC = AUDIO /"opengameart-rising-moon-randommind-cc0-224.7s.mp3"
BED_OUT = AUDIO / "reel-twin-moons-bed-rising-moon-from-4.92s-with-hits-27s.wav"
MUSIC_FROM, BED_SECONDS = 4.92, 27.0
WHOOSH_AT, IMPACT_AT = 22.30, 23.34  # reel seconds: into the launch (cut at 22.47), the impact frame


def ffmpeg(*args: str) -> None:
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *args], check=True)


def look_shot(out: Path) -> None:
    ins: list[str] = []
    parts = []
    for i, (png, n) in enumerate(EDITOR_STEPS):
        ins += ["-loop", "1", "-framerate", str(FPS), "-i", str(CLIPS / png)]
        parts.append(f"[{i}:v]scale={W}:{H}:flags=lanczos,setsar=1,trim=end_frame={n},setpts=PTS-STARTPTS,format=yuv420p[e{i}]")
    k = len(EDITOR_STEPS)
    ins += ["-i", str(TOWN_COLD), "-i", str(TOWN_ORANGE)]
    # the orange look comes in from the right (the moon's side): a hard wipe edge moving right to left
    x = f"{W}*(1-clip((N-{WIPE_AT})/{WIPE_FRAMES},0,1))"
    parts.append(f"[{k}:v]trim=end_frame={TOWN_FRAMES},setpts=PTS-STARTPTS,format=rgb24[cold]")
    parts.append(f"[{k + 1}:v]trim=end_frame={TOWN_FRAMES},setpts=PTS-STARTPTS,format=rgb24[warm]")
    parts.append(f"[cold][warm]blend=all_expr='if(gte(X,{x}),B,A)',format=yuv420p[town]")
    parts.append("".join(f"[e{i}]" for i in range(k)) + f"[town]concat=n={k + 1}:v=1:a=0,fps={FPS}[v]")
    ffmpeg(*ins, "-filter_complex", ";".join(parts), "-map", "[v]", "-c:v", "libx264", "-crf", "16", "-pix_fmt", "yuv420p", str(out))


def title_piece(out: Path) -> None:
    """The burst with the title cut in on it, small in the lower right (cream on a soft dark shadow, no box), the
    picture fading to black under it and the title held on black. (The cut's own `title` on a recording shot fails in
    the engine at 044b965: its work folder is not made before the title is drawn.)"""
    png = out.with_suffix(".title.png")
    t = ["-font", str(FONT), "-pointsize", "56", "-gravity", "southeast"]
    subprocess.run(["magick", "-size", f"{W}x{H}", "xc:none",
                    "(", "-size", f"{W}x{H}", "xc:none", *t, "-fill", "#0a081e", "-stroke", "#0a081e", "-strokewidth", "8", "-annotate", "+94+150", TITLE, "-blur", "0x9", ")",
                    "-composite", *t, "-stroke", "none", "-strokewidth", "0", "-fill", "#faf4e8", "-annotate", "+96+153", TITLE, str(png)], check=True)
    n = 48 - IMPACT_FRAME  # the clip's frames from the burst
    vf = (f"[0:v]trim=start_frame={IMPACT_FRAME},setpts=PTS-STARTPTS,fps={FPS},fade=t=out:st={(n - FADE) / FPS:.4f}:d={FADE / FPS:.4f},"
          f"tpad=stop={BLACK}:stop_mode=add:color=black[b];[b][1:v]overlay=0:0:shortest=1,format=yuv420p[v]")
    ffmpeg("-i", str(TITLE_CLIP), "-loop", "1", "-framerate", str(FPS), "-t", f"{(n + BLACK) / FPS:.4f}", "-i", str(png),
           "-filter_complex", vf, "-map", "[v]", "-frames:v", str(n + BLACK), "-c:v", "libx264", "-crf", "16", "-pix_fmt", "yuv420p", str(out))
    png.unlink()


def bed(out: Path) -> None:
    # whoosh: pink noise, band-limited, swelling for 0.45 s and dying in 0.2 s; thump: a sine falling 130 -> 42 Hz
    # with a fast decay, and a short low-passed noise burst on top
    whoosh = "anoisesrc=color=pink:amplitude=0.5:duration=0.65:sample_rate=48000,highpass=f=250,lowpass=f=3500,afade=t=in:d=0.45:curve=exp,afade=t=out:st=0.45:d=0.2,volume=0.7"
    thump = "aevalsrc='0.45*sin(2*PI*(42*t+88*(1-exp(-t*18))/18))*exp(-t*6)':d=0.9:s=48000"
    burst = "anoisesrc=color=brown:amplitude=0.6:duration=0.4:sample_rate=48000,lowpass=f=1200,afade=t=out:d=0.4:curve=exp,volume=0.3"
    fc = (
        f"[0:a]atrim=start={MUSIC_FROM}:duration={BED_SECONDS},asetpts=PTS-STARTPTS,aresample=48000,aformat=channel_layouts=stereo[m];"
        f"{whoosh},aformat=channel_layouts=stereo,adelay={int(WHOOSH_AT * 1000)}:all=1[w];"
        f"{thump},aformat=channel_layouts=stereo,adelay={int(IMPACT_AT * 1000)}:all=1[t];"
        f"{burst},aformat=channel_layouts=stereo,adelay={int(IMPACT_AT * 1000)}:all=1[b];"
        "[m][w][t][b]amix=inputs=4:duration=first:normalize=0,alimiter=limit=0.89[a]"
    )
    ffmpeg("-i", str(MUSIC), "-filter_complex", fc, "-map", "[a]", "-c:a", "pcm_s16le", "-ar", "48000", str(out))


if __name__ == "__main__":
    look_shot(LOOK_OUT)
    title_piece(TITLE_OUT)
    bed(BED_OUT)
    print(f"made {LOOK_OUT.relative_to(ROOT)}, {TITLE_OUT.relative_to(ROOT)} and {BED_OUT.relative_to(ROOT)}")
