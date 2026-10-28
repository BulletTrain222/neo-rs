"""Load every world pack headless from each of its first entry points, walk and jump for N
ticks, and flag crashes, falls through the world, NaN positions and errors. A run is judged
only up to its first game over or area change. The headless trace is not fully
deterministic (repeated runs of one build can take different paths, most likely from region
loads finishing on different ticks), so compare flagged runs across repeats before calling one
a regression. Usage: python sweep.py <repo> <ticks> [packs...]"""
import re, subprocess, sys, json, os, time

repo = sys.argv[1]
ticks = int(sys.argv[2])
# NEOPETS_SWEEP_EXE: run a copy of the exe, so the build can go on while the sweep runs.
exe = os.environ.get('NEOPETS_SWEEP_EXE') or os.path.join(repo, 'target', 'release', 'neopets-app.exe')
packs = sys.argv[3:]
MAX_ENTRIES = 4
if not packs:
    out = subprocess.run([exe, '--play-trace', '1', '--pack', '909'], cwd=repo, capture_output=True, text=True, env={**os.environ, 'NEOPETS_LIST_WORLDS': '1'}).stdout
    packs = re.search(r'WORLDS (.*)', out).group(1).split()
pos_re = re.compile(r'^\s*(\d+) pos \(\s*(\S+)\s+(\S+)\s+(\S+)\) state\s+(\d+).*?gap\s+(\S+)\s+vy\s+(\S+)')
results = []
os.makedirs(os.path.join(repo, 'private', 'sweep'), exist_ok=True)


def run(p, entry):
    env = dict(os.environ)
    if entry is not None:
        env['NEOPETS_TRACE_ENTRY'] = str(entry)
    try:
        r = subprocess.run([exe, '--play-trace', str(ticks), '--pack', p, '--autojump'], cwd=repo, capture_output=True, text=True, timeout=600, errors='replace', env=env)
        return r.returncode, r.stdout + r.stderr
    except subprocess.TimeoutExpired:
        return 'timeout', ''


def judge(code, out):
    rows = []
    for line in out.splitlines():
        if 'game over' in line or re.search(r'^\s*loaded \d+ on tick \d+ .*door request true', line):
            break
        m = pos_re.match(line)
        if m:
            rows.append((int(m.group(1)), float(m.group(2)), float(m.group(3)), float(m.group(4)), int(m.group(5)), float(m.group(6)), float(m.group(7))))
    issues = []
    if code != 0:
        issues.append(f'exit {code}')
    if 'panicked' in out:
        issues.append('PANIC: ' + next(l for l in out.splitlines() if 'panicked' in l)[:200])
    if any(x != x for row in rows for x in row[1:4]):
        issues.append('NaN position')
    if rows:
        # A fall through the world: gap far beyond any floor (no ground under the hero) while
        # falling fast for a second or more.
        streak = 0
        for r in rows:
            streak = streak + 1 if (r[5] > 1e6 and r[6] < -20) else 0
            if streak >= 60:
                issues.append(f'fell out of the world at tick {r[0]} near ({r[1]:.0f},{r[2]:.0f},{r[3]:.0f})')
                break
        moved = max(((r[1] - rows[0][1]) ** 2 + (r[3] - rows[0][3]) ** 2) ** 0.5 for r in rows)
        if moved < 1.0 and len(rows) > 600:
            issues.append(f'never moved (state {rows[-1][4]})')
    elif code == 0:
        issues.append('no position lines')
    return issues


for p in packs:
    t0 = time.time()
    code, out = run(p, None)
    n = int((re.search(r'Start points: (\d+)', out) or [0, 0])[1])
    entries = list(range(min(n, MAX_ENTRIES))) or [None]
    for e in entries:
        if e is not None:
            code, out = run(p, e)
        open(os.path.join(repo, 'private', 'sweep', f'trace_{p}_{e}.txt'), 'w', encoding='utf-8').write(out)
        issues = judge(code, out)
        results.append({'pack': p, 'entry': e, 'issues': issues})
        print(p, e, '; '.join(issues) or 'ok', flush=True)
json.dump(results, open(os.path.join(repo, 'private', 'sweep', 'results.json'), 'w'), indent=1)
