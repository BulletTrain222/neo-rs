"""Walk every world pack's townsfolk for a while and report how they walk. A headless pass per pack,
hero standing, every embedded record bound (NEOPETS_NO_STREAM), with NEOPETS_TRACE_NPCWALK. Written to
private/npc-walk/<tag>.txt, one line per walker, with a summary at the end. A flag is a lead, not proof
of a fault. Usage: python npc_walk_check.py <repo> <tag> [workers] [ticks] [packs...]"""
import os, re, shutil, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

repo, tag = sys.argv[1], sys.argv[2]
workers = int(sys.argv[3]) if len(sys.argv) > 3 else 6
ticks = sys.argv[4] if len(sys.argv) > 4 else '3600'
out = os.path.join(repo, 'private', 'npc-walk')
os.makedirs(out, exist_ok=True)
exe = os.path.join(out, 'check.exe')
shutil.copy(os.path.join(repo, 'target', 'release', 'neopets-app.exe'), exe)
packs = sys.argv[5:]
if not packs:
    t = subprocess.run([exe, '--play-trace', '1', '--pack', '909'], cwd=repo, capture_output=True, text=True, errors='replace', env={**os.environ, 'NEOPETS_LIST_WORLDS': '1'}).stdout
    packs = re.search(r'WORLDS (.*)', t).group(1).split()
LINE = re.compile(r'NPCWALK pack (\d+) "(.*?)" code (-?\d+) slot (\d+) ordinal (\d+) walked ([\d.]+) in (\d+) ticks, wall steps (\d+), off mesh (\d+), longest still (\d+), route legs (\d+) crossing (\d+) off (\d+), at (.*)')


def run(p):
    env = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}
    env.update({'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_NO_STREAM': '1', 'NEOPETS_TRACE_NPCWALK': str(int(ticks) - 1)})
    try:
        r = subprocess.run([exe, '--play-trace', ticks, '--pack', p, '--seed', '1'], cwd=repo, capture_output=True, text=True, errors='replace', env=env, timeout=240)
        text, code = r.stdout + r.stderr, r.returncode
    except subprocess.TimeoutExpired:
        return p, 'timeout', []
    return p, code, [l.strip() for l in text.splitlines() if l.startswith('NPCWALK') or 'panicked' in l]


with ThreadPoolExecutor(workers) as ex:
    res = list(ex.map(run, packs))
lines, bad = [], []
for p, code, ls in res:
    if code != 0:
        bad.append((p, code))
    lines += ls
rows = [m.groups() for m in map(LINE.search, lines) if m]
walkers = [r for r in rows if r[2] in ('5', '6')]
never = [r for r in walkers if float(r[5]) < 1.0]
walls = [r for r in walkers if int(r[7]) > 0]
off = [r for r in walkers if int(r[8]) > 60]
still = [r for r in walkers if int(r[9]) > 600]
routes = [r for r in walkers if int(r[11]) > 0 or int(r[12]) > 0]
with open(os.path.join(out, tag + '.txt'), 'w', encoding='utf-8') as f:
    f.write('\n'.join(lines) + '\n')
    s = f'packs {len(packs)}, failures {bad}\nwalkers {len(walkers)} (wander {sum(r[2] == "5" for r in rows)}, patrol {sum(r[2] == "6" for r in rows)})\nnever moved {len(never)}\nwalked through walls {len(walls)}\noff the mesh over 1 s {len(off)}\nstill over 10 s while asked to walk {len(still)}\nroutes with a leg through a wall or a point off the mesh {len(routes)}\n'
    f.write(s)
print(s)
