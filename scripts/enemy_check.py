"""Check every common enemy's attacks over every world pack. Three headless passes with god mode:
a fight (the duel trace, the hero attacking each enemy in turn) and two passive ones (the hero
placed 14 or 4 away, standing still). NEOPETS_TRACE_ATTACKS logs each notice, attack choice,
strike and shot with its distance. Grouped by enemy in private/enemy-check/report.txt. A flag is a
lead to look at, not proof of a fault. Usage: python enemy_check.py <repo> [workers] [packs...];
CHECK_REPORT_ONLY=1 rebuilds the report from the kept traces."""
import os, re, subprocess, sys, shutil
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor

repo = sys.argv[1]
workers = int(sys.argv[2]) if len(sys.argv) > 2 else 6
out = os.path.join(repo, 'private', 'enemy-check')
os.makedirs(out, exist_ok=True)
PASSES = {'fight': {'NEOPETS_TRACE_DUEL': '900'}, 'far': {'NEOPETS_TRACE_DUEL': '500', 'NEOPETS_TRACE_DUEL_PASSIVE': '14'}, 'near': {'NEOPETS_TRACE_DUEL': '400', 'NEOPETS_TRACE_DUEL_PASSIVE': '4'}}
# A copy of the exe, so the build can go on while this runs.
exe = os.path.join(out, 'check.exe')


def run(job):
    p, name = job
    path = os.path.join(out, f'{name}_{p}.txt')
    env = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}
    env.update({'NEOPETS_TRACE_ATTACKS': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_STAND': '1', **PASSES[name]})
    try:
        r = subprocess.run([exe, '--play-trace', '60000', '--pack', p, '--give', '105', '--seed', '1'], cwd=repo, capture_output=True, text=True, timeout=1500, errors='replace', env=env)
        text, code = r.stdout + r.stderr, r.returncode
    except subprocess.TimeoutExpired as e:
        got = e.stdout or b''
        text, code = got.decode('utf-8', 'replace') if isinstance(got, bytes) else got, 'timeout'
    keep = [l for l in text.splitlines() if l.startswith(('ATK ', 'DUEL ')) or 'panicked' in l]
    open(path, 'w', encoding='utf-8').write(f'# exit {code}\n' + '\n'.join(keep) + '\n')
    print(name, p, code, len(keep), flush=True)


if not os.environ.get('CHECK_REPORT_ONLY'):
    shutil.copy(os.path.join(repo, 'target', 'release', 'neopets-app.exe'), exe)
    packs = sys.argv[3:]
    if not packs:
        t = subprocess.run([exe, '--play-trace', '1', '--pack', '909'], cwd=repo, capture_output=True, text=True, errors='replace', env={**os.environ, 'NEOPETS_LIST_WORLDS': '1'}).stdout
        packs = re.search(r'WORLDS (.*)', t).group(1).split()
    with ThreadPoolExecutor(workers) as ex:
        list(ex.map(run, [(p, n) for n in PASSES for p in packs]))

E = defaultdict(lambda: {'notice': [], 'select': defaultdict(list), 'strike': defaultdict(list), 'shot': defaultdict(list), 'band': {}, 'kind': {}, 'dmg': {}, 'mis': {}, 'far': 0, 'near': 0, 'killed': 0, 'duels': 0, 'flags': defaultdict(list), 'packs': set()})
bad = []
slot_key = {}
for f in sorted(os.listdir(out)):
    if not f.endswith('.txt') or f == 'report.txt':
        continue
    pas, pack = f[:-4].split('_', 1)
    for l in open(os.path.join(out, f), encoding='utf-8'):
        if l.startswith('# exit') and l.split()[2] != '0':
            bad.append(f'{f}: exit {l.split()[2]}')
        if 'panicked' in l:
            bad.append(f'{f}: {l.strip()[:150]}')
        m = re.match(r'ATK (\w+) "([^"]*)" (.*)', l)
        if m:
            k, name, rest = m.groups()
            e = E[name]
            e['packs'].add(pack)
            v = rest.split()
            g = {key: v[i + 1] for i, key in enumerate(v[:-1]) if not key[0].isdigit()}
            if k == 'notice':
                factor = float(v[v.index('bands') + 1]) if 'bands' in v else 1.0
                e['notice'].append((float(g['d']), float(g['near']), float(g['far']) * max(factor, 1.0), g.get('alarm', 'None') != 'None'))
            elif k == 'select':
                i = v.index('band')
                band = (float(v[i + 1]), float(v[i + 2]))
                # One name can cover rows with different slots: a slot is its number and band.
                s = f"{g['slot']} ({band[0]:g} to {band[1]:g})"
                slot_key[(name, g['slot'])] = s
                e['select'][s].append(float(g['d']))
                e['band'][s] = band
                e['kind'][s] = g['kind']
                e['dmg'][s] = g['damage']
            elif k == 'strike':
                e['strike'][slot_key.get((name, g['slot']), g['slot'])].append((float(g['d']), float(g['h']), float(g['radius'])))
            elif k == 'shot':
                key = slot_key.get((name, g['slot']), g['slot'])
                e['shot'][key].append(float(g['d']))
                e['mis'][key] = (g['speed'], g['range'], g['flight'], g['damage'])
        m = re.match(r'DUEL (\d+) (\d+) "([^"]*)" hp (-?\d+)/(\d+) (killed|alive) .*? states \{[^}]*\} (\S+)', l)
        if m and pas == 'fight':
            e = E[m[3]]
            e['duels'] += 1
            e['packs'].add(m[1])
            e['killed'] += m[6] == 'killed'
            for fl in m[7].split(','):
                if fl != 'ok':
                    e['flags'][fl].append(f'{m[1]}#{m[2]}')
        elif m:
            E[m[3]][pas] += 1
            E[m[3]]['packs'].add(m[1])
rows, leads = [], []
for name in sorted(E):
    e = E[name]
    tries = e['far'] + e['near'] + e['duels']
    rows.append(f"\n{name}: seen in {len(e['packs'])} packs; fights {e['killed']}/{e['duels']} killed; stood by far {e['far']} near {e['near']}; flags { {k: len(v) for k, v in e['flags'].items()} or 'none'}")
    if e['notice']:
        ds = [n[0] for n in e['notice']]
        over = [n for n in e['notice'] if n[0] > n[2] + 0.5 and not n[3]]
        rows.append(f"  notice {len(ds)}x at {min(ds):.1f} to {max(ds):.1f} (row near {e['notice'][0][1]:.1f}, far {e['notice'][0][2]:.1f})" + (f"; {len(over)} past far" if over else ''))
        if over:
            leads.append(f"{name}: noticed past its far range {len(over)}x (largest {max(n[0] for n in over):.1f}, far {over[0][2]:.1f})")
    elif tries and not e['select']:
        leads.append(f"{name}: never noticed the hero in {tries} tries")
    if not e['select'] and tries:
        leads.append(f"{name}: never chose an attack in {tries} tries")
    for s in sorted(e['select']):
        ds = e['select'][s]
        lo, hi = e['band'][s]
        out_band = [d for d in ds if not lo - 0.01 <= d <= hi + 0.01]
        line = f"  slot {s} kind {e['kind'][s]} band {lo:.1f} to {hi:.1f} damage {e['dmg'][s]}: chosen {len(ds)}x at {min(ds):.1f} to {max(ds):.1f}"
        if out_band:
            line += f"; {len(out_band)} OUTSIDE BAND"
            leads.append(f"{name} slot {s}: chosen outside its band {len(out_band)}x")
        st = e['strike'].get(s, [])
        if st:
            line += f"; strikes {len(st)}x at {min(x[0] for x in st):.1f} to {max(x[0] for x in st):.1f}"
            wide = max(x[1] for x in st)
            if wide > hi + 4:
                leads.append(f"{name} slot {s}: a strike landed {wide:.1f} away, band ends at {hi:.1f}")
        elif e['kind'][s] == '0' and len(ds) >= 8:
            leads.append(f"{name} slot {s}: melee chosen {len(ds)}x, never connected")
        sh = e['shot'].get(s, [])
        if sh:
            sp, rg, fl, dm = e['mis'][s]
            line += f"; shots {len(sh)}x at {min(sh):.1f} to {max(sh):.1f} (speed {sp}, range {rg}, flight {fl}, damage {dm})"
        elif e['kind'][s] == '1' and len(ds) >= 8:
            leads.append(f"{name} slot {s}: missile chosen {len(ds)}x, never fired")
        rows.append(line)
    for fl, where in e['flags'].items():
        leads.append(f"{name}: {fl} in {len(where)} of {e['duels']} fights ({' '.join(where[:5])})")
    if e['duels'] and e['killed'] == 0 and not e['flags']:
        leads.append(f"{name}: never killed in {e['duels']} fights")
text = f"{len(E)} enemy names. Runs with problems: {len(bad)}\n" + '\n'.join(bad) + "\n\nLEADS\n" + '\n'.join(leads) + "\n\nDETAIL" + '\n'.join(rows) + '\n'
open(os.path.join(out, 'report.txt'), 'w', encoding='utf-8').write(text)
print(text[:7000])
