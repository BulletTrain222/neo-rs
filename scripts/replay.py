"""Replays a bug report's last minute of input in a headless trace.

    python scripts/replay.py 12          report 12: writes private/reports/012/replay.log
    python scripts/replay.py 12 FACING=Brendel    with a NEOPETS_TRACE_ switch

It starts from the report's snapshot (world, hero position, flags, items), which is taken every 30 seconds. Enemies and scripts
are not in the snapshot, so a replay comes close to what happened; it is not exact.
"""
import os, re, subprocess, sys

repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
os.chdir(repo)
n = int(sys.argv[1])
d = os.path.join('private', 'reports', f'{n:03}')
start = dict(l.split(' ', 1) for l in open(os.path.join(d, 'start.txt')).read().splitlines() if ' ' in l)
rows = sum(1 for _ in open(os.path.join(d, 'input.csv')))
env = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}
env['NEOPETS_REPLAY'] = d
for a in sys.argv[2:]:
    k, _, v = a.partition('=')
    env['NEOPETS_TRACE_' + k] = v or '1'
cmd = [os.path.join('target', 'release', 'neopets-app.exe'), '--play-trace', str(rows + 30), '--pack', start['pack'], '--seed', '1']
r = subprocess.run(cmd, env=env, capture_output=True, text=True, errors='replace', stdin=subprocess.DEVNULL)
log = os.path.join(d, 'replay.log')
open(log, 'w', encoding='utf-8').write(r.stdout + r.stderr)
pos = re.findall(r'^\s*\d+ pos .*$', r.stdout, re.M)
print(f'{rows} ticks replayed from tick {start.get("tick")} in world {start["pack"]}; log in {log}')
print('last position:', pos[-1].strip()[:90] if pos else 'none')
if 'panicked' in r.stderr:
    print('THE GAME PANICKED:', r.stderr[r.stderr.index('panicked'):][:300])
