"""Fight every common enemy of every world pack with the duel trace and group the outcomes by
enemy name. Each pack runs headless from its first entry with Tor holding the sword (item 99),
god mode on, and NEOPETS_TRACE_DUEL placing him beside each living hostile common enemy in turn
(each given LIMIT ticks). Flags: never-hurt (its HP never fell), not-beaten (hurt but alive at
the limit), froze (state, sub-states, clip and position unchanged for 300 ticks while the hero
was close), fell (30 below its home), never-reached. A flag is a lead to look at, not proof of
a game fault: the trace fights crudely. Usage: python enemy_sweep.py <repo> [limit] [packs...]"""
import json, os, re, subprocess, sys
from collections import defaultdict

repo = sys.argv[1]
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 1800
# NEOPETS_SWEEP_EXE: run a copy of the exe, so the build can go on while the sweep runs.
exe = os.environ.get('NEOPETS_SWEEP_EXE') or os.path.join(repo, 'target', 'release', 'neopets-app.exe')
packs = sys.argv[3:]
if not packs:
    out = subprocess.run([exe, '--play-trace', '1', '--pack', '909'], cwd=repo, capture_output=True, text=True, env={**os.environ, 'NEOPETS_LIST_WORLDS': '1'}).stdout
    packs = re.search(r'WORLDS (.*)', out).group(1).split()
line_re = re.compile(r'^DUEL (\d+) (\d+) "([^"]*)" hp (-?\d+)/(\d+) (killed|alive) in (\d+) ticks engaged (\d+) still (\d+) blocks (\d+) states (\{[^}]*\}) (.*)$')
out_dir = os.path.join(repo, 'private', 'enemy-sweep')
os.makedirs(out_dir, exist_ok=True)
fights = []
for p in packs:
    env = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}
    env.update({'NEOPETS_TRACE_DUEL': str(limit), 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_STAND': '1'})
    try:
        r = subprocess.run([exe, '--play-trace', '60000', '--pack', p, '--give', '99', '--seed', '1'], cwd=repo, capture_output=True, text=True, timeout=1800, errors='replace', env=env)
        text = r.stdout + r.stderr
        code = r.returncode
    except subprocess.TimeoutExpired:
        text, code = '', 'timeout'
    open(os.path.join(out_dir, f'trace_{p}.txt'), 'w', encoding='utf-8').write(text)
    found = 0
    for line in text.splitlines():
        m = line_re.match(line)
        if m:
            found += 1
            fights.append({'pack': int(m[1]), 'id': int(m[2]), 'name': m[3], 'hp': int(m[4]), 'max': int(m[5]), 'killed': m[6] == 'killed', 'ticks': int(m[7]), 'engaged': int(m[8]), 'still': int(m[9]), 'blocks': int(m[10]), 'states': m[11], 'flags': m[12]})
    crash = 'panicked' in text or code not in (0,)
    print(p, f'{found} enemies', 'CRASH' if crash else '', flush=True)
json.dump(fights, open(os.path.join(out_dir, 'fights.json'), 'w'), indent=1)
by = defaultdict(list)
for f in fights:
    by[f['name']].append(f)
lines = []
for name in sorted(by):
    fs = by[name]
    killed = sum(f['killed'] for f in fs)
    flags = defaultdict(list)
    for f in fs:
        for fl in f['flags'].split(','):
            if fl != 'ok':
                flags[fl].append(f"{f['pack']}#{f['id']}")
    avg = sum(f['ticks'] for f in fs if f['killed']) / max(killed, 1)
    lines.append(f"{name}: {killed}/{len(fs)} killed, {avg:.0f} ticks each" + ''.join(f"; {k} {len(v)} ({' '.join(v[:6])})" for k, v in sorted(flags.items())))
open(os.path.join(out_dir, 'summary.txt'), 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print('\n'.join(lines))
