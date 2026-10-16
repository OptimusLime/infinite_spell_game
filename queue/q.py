#!/usr/bin/env python3
"""The work queue for the Infinite Spell Game and the Atelico engine work behind it.

One SQLite file (queue/queue.db) is the source of truth, owned by the engine's atelico-queue crate; queue/queue.md is
a readable snapshot written after every change. Every agent reads its task here and writes its progress here.

This script is now a thin wrapper: it runs the engine's `app queue` with the same arguments, so the old habits keep
working and gain the new verbs. Use `app queue` directly when you can (see the engine's docs/queue.md):

  q.py list [--all]                      open tasks by priority (--all: done and dropped too)
  q.py show ID                           one task with its history
  q.py add TITLE --track T --lane L [--owner O] [--priority N] [--detail D] [--quote Q] [--group G] [--tag T] [--size S]
  q.py set ID [--status S] [--owner O] [--priority N] [--detail D] [--commits C] [--evidence E] [--group G] ...
  q.py note ID TEXT [--who W]            add to a task's history
  q.py open --json                       the open tasks, every field
  q.py search "TEXT"                     open tasks by meaning (by words when the AI engine is down)
  q.py next [--lane L] [--about "what I am good at"] [--claim --who W]
  q.py claim ID --who W                  take a todo; atomic, one agent wins
  q.py group suggest | apply NAME ID... | list

Only when the engine's `app` binary is missing does it fall back to the old built-in commands below (list, show,
add, set, note), which still work against the migrated file.
"""
import argparse, datetime, os, sqlite3, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
DB = os.path.join(HERE, "queue.db")
SNAPSHOT = os.path.join(HERE, "queue.md")
STATUSES = ("todo", "doing", "blocked", "review", "done", "dropped")
TRACKS = ("pieces", "software")
MARK = {"todo": "⚪", "doing": "🟡", "blocked": "🔴", "review": "🔵", "done": "🟢", "dropped": "⚫"}

SCHEMA = f"""
CREATE TABLE IF NOT EXISTS tasks (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  track TEXT NOT NULL CHECK (track IN {TRACKS}),
  lane TEXT NOT NULL,
  owner TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'todo' CHECK (status IN {STATUSES}),
  priority INTEGER NOT NULL DEFAULT 50,
  detail TEXT NOT NULL DEFAULT '',
  quote TEXT NOT NULL DEFAULT '',
  commits TEXT NOT NULL DEFAULT '',
  evidence TEXT NOT NULL DEFAULT '',
  created TEXT NOT NULL,
  updated TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY,
  task INTEGER NOT NULL REFERENCES tasks(id),
  at TEXT NOT NULL,
  who TEXT NOT NULL,
  text TEXT NOT NULL
);
"""

def now():
    return datetime.datetime.now().isoformat(timespec="seconds")

def db():
    c = sqlite3.connect(DB, timeout=30)
    c.row_factory = sqlite3.Row
    c.execute("PRAGMA journal_mode=WAL")
    c.executescript(SCHEMA)
    return c

def log(c, task, who, text):
    c.execute("INSERT INTO events (task, at, who, text) VALUES (?, ?, ?, ?)", (task, now(), who, text))

def snapshot(c):
    rows = c.execute("SELECT * FROM tasks ORDER BY status IN ('done','dropped'), priority, id").fetchall()
    out = ["# Queue", "", f"Snapshot of queue/queue.db, {now()}. Edit through `queue/q.py`, not this file.", ""]
    for track in TRACKS:
        out += [f"## {track.capitalize()}", "", "| # | | Task | Lane | Owner | P | Commits |", "|---|---|---|---|---|---|---|"]
        for r in rows:
            if r["track"] == track:
                out.append(f"| {r['id']} | {MARK[r['status']]} {r['status']} | {r['title']} | {r['lane']} | {r['owner']} | {r['priority']} | {r['commits']} |")
        out.append("")
    open(SNAPSHOT, "w").write("\n".join(out))

def legacy_main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    s = p.add_subparsers(dest="cmd", required=True)
    l = s.add_parser("list"); l.add_argument("--all", action="store_true")
    sh = s.add_parser("show"); sh.add_argument("id", type=int)
    a = s.add_parser("add"); a.add_argument("title")
    a.add_argument("--track", required=True, choices=TRACKS); a.add_argument("--lane", required=True)
    a.add_argument("--owner", default=""); a.add_argument("--priority", type=int, default=50)
    a.add_argument("--detail", default=""); a.add_argument("--quote", default=""); a.add_argument("--status", default="todo", choices=STATUSES)
    st = s.add_parser("set"); st.add_argument("id", type=int)
    for f in ("owner", "detail", "commits", "evidence", "title", "lane"): st.add_argument(f"--{f}")
    st.add_argument("--status", choices=STATUSES); st.add_argument("--priority", type=int); st.add_argument("--who", default="coordinator")
    n = s.add_parser("note"); n.add_argument("id", type=int); n.add_argument("text"); n.add_argument("--who", default="coordinator")
    args = p.parse_args()
    c = db()
    if args.cmd == "list":
        q = "SELECT * FROM tasks" + ("" if args.all else " WHERE status NOT IN ('done','dropped')") + " ORDER BY track, priority, id"
        for r in c.execute(q):
            print(f"{r['id']:>3} {MARK[r['status']]} {r['status']:<7} P{r['priority']:<3} [{r['track']}/{r['lane']}] {r['title']}  ({r['owner'] or '-'})")
        return
    if args.cmd == "show":
        r = c.execute("SELECT * FROM tasks WHERE id=?", (args.id,)).fetchone()
        if not r: sys.exit(f"no task {args.id}")
        for k in r.keys(): print(f"{k:>9}: {r[k]}")
        for e in c.execute("SELECT * FROM events WHERE task=? ORDER BY id", (args.id,)): print(f"  {e['at']} {e['who']}: {e['text']}")
        return
    with c:
        if args.cmd == "add":
            t = now()
            cur = c.execute("INSERT INTO tasks (title, track, lane, owner, status, priority, detail, quote, created, updated) VALUES (?,?,?,?,?,?,?,?,?,?)",
                            (args.title, args.track, args.lane, args.owner, args.status, args.priority, args.detail, args.quote, t, t))
            log(c, cur.lastrowid, "coordinator", f"added ({args.status})"); print(cur.lastrowid)
        elif args.cmd == "set":
            changes = {f: getattr(args, f) for f in ("status", "owner", "priority", "detail", "commits", "evidence", "title", "lane") if getattr(args, f) is not None}
            if not changes: sys.exit("nothing to set")
            c.execute(f"UPDATE tasks SET {', '.join(f'{k}=?' for k in changes)}, updated=? WHERE id=?", (*changes.values(), now(), args.id))
            log(c, args.id, args.who, ", ".join(f"{k}={v}" for k, v in changes.items()))
        elif args.cmd == "note":
            c.execute("UPDATE tasks SET updated=? WHERE id=?", (now(), args.id)); log(c, args.id, args.who, args.text)
        snapshot(c)


ENGINE = os.path.expanduser("~/coding/atelico/atelico-app-engine")
APP = [os.path.join(ENGINE, "target", "debug", "app"), os.path.join(ENGINE, "target", "release", "app")]
GAME = os.path.dirname(HERE)


def app():
    """The newest `app` binary that knows `queue`, or None."""
    found = [p for p in APP if os.path.exists(p)]
    found.sort(key=os.path.getmtime, reverse=True)
    for p in found:
        if subprocess.run([p, "queue", "--help"], capture_output=True).returncode == 0:
            return p
    return None


def main():
    a = app()
    if a is None:
        print("(engine `app` not built with `queue`: using the built-in commands)", file=sys.stderr)
        return legacy_main()
    # run from the game repo: its atelico.toml names queue/queue.db and queue/queue.md
    sys.exit(subprocess.run([a, "queue", *sys.argv[1:]], cwd=GAME).returncode)


if __name__ == "__main__":
    main()
