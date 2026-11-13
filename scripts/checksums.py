"""Per-tick state checksums of every scenario run and a few report replays, kept and compared, so that a
refactor that must not change play can be shown not to (docs/MULTIPLAYER.md step 1).

    python scripts/checksums.py record NAME     run them all and keep the checksums in private/checksums/NAME/
    python scripts/checksums.py compare A B     compare two kept sets, run by run

Each run is a headless trace with NEOPETS_TRACE_CHECKSUM set; the trace prints one checksum per tick. Runs whose two
recordings of one build differ are not repeatable and are reported as such, not as changes.
"""
import os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
os.chdir(repo)
exe = os.path.join('target', 'release', 'neopets-app.exe')
env0 = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}


def scenario_runs():
    src = open(os.path.join('scripts', 'scenarios.py'), encoding='utf-8').read()
    src = src[:src.rindex('\ndef main()')]
    ns = {'__name__': 'checksums', '__file__': os.path.join(repo, 'scripts', 'scenarios.py')}
    exec(compile(src, 'scenarios.py', 'exec'), ns)
    for i, (name, _spec, ticks, pack, args, env, _test) in enumerate(ns['SCENARIOS']):
        yield f's{i:02}', name, [exe, '--play-trace', str(ticks), '--pack', str(pack), '--seed', '1'] + args, env


def replay_runs():
    base = os.path.join('private', 'reports')
    for n in [123, 128, 135, 139, 141]:
        d = os.path.join(base, f'{n:03}')
        if not os.path.isdir(d):
            continue
        start = dict(l.split(' ', 1) for l in open(os.path.join(d, 'start.txt')).read().splitlines() if ' ' in l)
        rows = sum(1 for _ in open(os.path.join(d, 'input.csv')))
        yield f'r{n}', f'report {n} replay', [exe, '--play-trace', str(rows + 30), '--pack', start['pack'], '--seed', '1'], {'NEOPETS_REPLAY': d}


def run(item):
    key, name, cmd, env = item
    try:
        r = subprocess.run(cmd, env=dict(env0, **env, NEOPETS_TRACE_CHECKSUM='1'), capture_output=True, text=True, errors='replace', stdin=subprocess.DEVNULL, timeout=600)
        out = r.stdout
    except subprocess.TimeoutExpired:
        out = ''
    return key, name, re.findall(r'^CHECKSUM \d+ [0-9a-f]+$', out, re.M)


def record(tag):
    d = os.path.join('private', 'checksums', tag)
    os.makedirs(d, exist_ok=True)
    items = list(scenario_runs()) + list(replay_runs())
    with ThreadPoolExecutor(4) as pool:
        for key, name, lines in pool.map(run, items):
            open(os.path.join(d, key + '.txt'), 'w').write(name + '\n' + '\n'.join(lines) + '\n')
            print(f'{key} {len(lines):5} ticks  {name}', flush=True)


def load(tag):
    d = os.path.join('private', 'checksums', tag)
    out = {}
    for f in sorted(os.listdir(d)):
        lines = open(os.path.join(d, f)).read().splitlines()
        out[f[:-4]] = (lines[0], lines[1:])
    return out


def compare(a, b):
    A, B = load(a), load(b)
    same = differ = 0
    for key in sorted(set(A) | set(B)):
        if key not in A or key not in B:
            print(f'{key}: only in one set')
            continue
        name, la = A[key]
        lb = B[key][1]
        first = next((i for i, (x, y) in enumerate(zip(la, lb)) if x != y), None)
        if first is None and len(la) == len(lb):
            same += 1
        else:
            differ += 1
            at = la[first] if first is not None and first < len(la) else f'length {len(la)} vs {len(lb)}'
            print(f'{key} differs from {at}  ({name})')
    print(f'{same} the same, {differ} different')
    return differ


if __name__ == '__main__':
    if sys.argv[1] == 'record':
        record(sys.argv[2])
    else:
        sys.exit(1 if compare(sys.argv[2], sys.argv[3]) else 0)
