"""Scripted checks of behaviour against the specs, each a short headless run of the game against the user's disc.

    python scripts/scenarios.py          run them all
    python scripts/scenarios.py brendel  run those whose name holds the word

Each scenario gives the world, the switches and a test of the run's output. Add one for every bug fixed, so that it stays
fixed. They need the ISO, so they are not part of Check.cmd.
"""
import os, re, subprocess, sys

repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
os.chdir(repo)
exe = os.path.join('target', 'release', 'neopets-app.exe')
env0 = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}


def last_facing(out):
    rows = re.findall(r'FACING (\d+) (-?[\d.]+) (-?[\d.]+) (-?[\d.]+) clip (\d+)', out)
    return rows[-1] if rows else None


def brendel(out):
    r = last_facing(out)
    if not r:
        return 'no Brendel in the run'
    x, z = float(r[1]), float(r[2])
    ok = abs(x + 679.37) < 0.5 and abs(z - 671.60) < 0.5 and r[4] == '0'
    return None if ok else f'rests at {x}, {z}, clip {r[4]}'


def count(pattern, want):
    def test(out):
        n = len(re.findall(pattern, out))
        return None if n == want else f'{n} matches of {pattern!r}, wanted {want}'
    return test


def has(pattern):
    def test(out):
        return None if re.search(pattern, out) else f'no match of {pattern!r}'
    return test


# name, spec, ticks, pack, extra arguments, environment, test.
SCENARIOS = [
    ('brendel rests at his centroid', 'SPEC-0056 TV-0056-R228a', 2600, 1066, ['--start', '-640,660,40'], {'NEOPETS_TRACE_FACING': 'Brendel'}, brendel),
    ('farm road spore monster grows nothing', 'SPEC-0064 Seed spawn step 8', 1500, 934, [], {'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_PLACE': '30:-1266,977,60'}, count(r'summoned key 619', 0)),
    ('rope gun from the farm plane', 'SPEC-0058 goal 354', 1500, 1217, ['--flag', '6', '--flag', '278', '--start', '5.5,45,-3'], {'NEOPETS_TRACE_PUSH': '5.5,52', 'NEOPETS_TRACE_GOD': '1'}, has(r'Held items at the end: .*\(36, 1\)')),
    ('sound records match the spec', 'SPEC-0031 types 4 and 5', 1, 909, [], {'NEOPETS_TRACE_CUE_RECORDS': '1'}, has(r'CUE RECORDS type 4 81, type 5 24')),
]


def main():
    words = sys.argv[1:]
    failed = 0
    for name, spec, ticks, pack, args, env, test in SCENARIOS:
        if words and not any(w in name for w in words):
            continue
        cmd = [exe, '--play-trace', str(ticks), '--pack', str(pack), '--seed', '1'] + args
        try:
            r = subprocess.run(cmd, env=dict(env0, **env), capture_output=True, text=True, errors='replace', stdin=subprocess.DEVNULL, timeout=400)
            problem = test(r.stdout + r.stderr)
            if 'panicked' in r.stderr:
                problem = 'the game panicked: ' + r.stderr[r.stderr.index('panicked'):][:200]
        except subprocess.TimeoutExpired:
            problem = 'timed out'
        print(('PASS' if problem is None else 'FAIL'), name, f'({spec})', '' if problem is None else '- ' + problem)
        failed += problem is not None
    sys.exit(1 if failed else 0)


main()
