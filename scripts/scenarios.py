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


def clear_of_pole(out):
    rows = re.findall(r'pos \(\s*(-?[\d.]+)\s+-?[\d.]+\s+(-?[\d.]+)\)', out)
    if not rows:
        return 'no positions in the run'
    nearest = min(((float(x) + 526.6) ** 2 + (float(z) - 623.0) ** 2) ** 0.5 for x, z in rows)
    return None if nearest >= 3.1 else f'hero came within {nearest:.2f} of the pole'


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
    ('father repairs the pump after the pliers', 'SPEC-0045 question 29', 1200, 909, ['--flag', '110', '--flag', '114', '--flag', '116', '--flag', '618'], {'NEOPETS_TRACE_FACING': 'Hubert', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_SETFLAG': '500:117'}, has(r'FACING \d+ \S+ \S+ \S+ clip 110')),
    ('a lock chest gives its item', 'R-0234', 200, 947, [], {'NEOPETS_TRACE_CHEST': '60:1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_STAND': '1'}, has(r'picked up item 55')),
    ('a sealed chest gives its item', 'R-0234', 200, 947, [], {'NEOPETS_TRACE_CHEST': '60:0', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_STAND': '1'}, has(r'picked up map 23')),
    ('no cloud wall before the flags (world 696)', 'SPEC-0042 696 Init; SPEC-0034 section 6', 420, 696, ['--start', '-286.86,670.9,45.2'], {'NEOPETS_TRACE_PLACED': '696:0-16', 'NEOPETS_TRACE_GOD': '1'}, has(r'PLACED tick 400 world 696: 0 active')),
    ('no cloud wall after walking in from 1066', 'SPEC-0042 696 Init; SPEC-0034 section 6', 420, 1066, [], {'NEOPETS_TRACE_PLACED': '696:0-16', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_PLACE': '100:-286.86,670.9,60'}, has(r'PLACED tick 400 world 696: 0 active')),
    ('the Meerca resumes his chase along the road', 'SPEC-0045 chase thread, question 27', 1500, 1023, ['--flag', '127', '--flag', '128', '--start', '-5,490,40'], {'NEOPETS_TRACE_FACING': 'Meerouladen', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_PLACE': '500:30,424,40'}, has(r'WALKSTATE 1400 .*mesh Some\(Some\(6\)\)')),
    ('flag poles stop the hero', 'SPEC-0018 Contact pass; R-0238', 200, 696, ['--start', '-535,615,44'], {'NEOPETS_TRACE_PUSH': '-526.6,623', 'NEOPETS_TRACE_GOD': '1'}, clear_of_pole),
    ('the hero walks onto the 1066 bridge', 'R-0237', 300, 1066, ['--flag', '17', '--start', '-652,685,44'], {'NEOPETS_TRACE_PUSH': '-705,745', 'NEOPETS_TRACE_GOD': '1'}, has(r'28\d pos \( -69\d')),
    ('a wild petpet eats and joins', 'SPEC-0047 Acquisition', 500, 1066, ['--start', '-759.5,545.5,40'], {'NEOPETS_TRACE_WILD': '100', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_GIVE': '5:150:5'}, has(r'joins with T')),
    ('Tor stands at his place at the pump', 'SPEC-0045 909 routine 11; R-0239', 300, 909, ['--flag', '1', '--flag', '107', '--flag', '108', '--flag', '109', '--flag', '110', '--flag', '111'], {'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GIVE': '5:6:1', 'NEOPETS_TRACE_PLACE': '60:-1949.41,1054.5,48', 'NEOPETS_TRACE_SQUARE': '100,101'}, has(r'28\d pos \(-1949\.\d+\s+45\.\d+\s+1055\.3')),
    ('a chest body survives the hero landing on it', 'SPEC-0048 Lock chests, Set-up', 300, 947, [], {'NEOPETS_TRACE_CHESTDROP': '60:1', 'NEOPETS_TRACE_GOD': '1'}, has(r'CHESTBODY 270: Some\(3\)')),
    ('the cloud wall shows once Act 2 starts', 'SPEC-0034 section 6, R-0235', 120, 696, ['--flag', '220', '--start', '-286.86,670.9,45.2'], {'NEOPETS_TRACE_PLACED': '696:0-16', 'NEOPETS_TRACE_GOD': '1'}, has(r'PLACED tick 100 world 696: 17 active')),
    ('carrying a Miamouse goes where the stick says', 'SPEC-0033 Behavior 18', 700, 1163, ['--flag', '156'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_CATCH': '100', 'NEOPETS_TRACE_SQUARE': '102,103,104,105,106,107,108,109,110,111,112', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_PUSH': '-80,70'}, has(r'CATCH 600: hero -(79|80)\.\d,(69|70)\.\d state 57')),
    ('800 basin ladder on object 0 climbs after the finale', 'SPEC-0028 7.2', 320, 800, ['--start', '472.33,2366.11,297', '--flag', '55'], {'NEOPETS_TRACE_TOWARD': '1:469.65,2361.9', 'NEOPETS_TRACE_STICK': '20:1,0', 'NEOPETS_TRACE_CLIMB': '1'}, has(r'300 pos \(\s+46\d\.\d+\s+305\.')),
    ('a ladder mounts from the floor and climbs', 'SPEC-0033 Behavior 13', 220, 1171, ['--start', '-374.98,3.12,-9.79'], {'NEOPETS_TRACE_CLIMB': '1'}, has(r'200 pos \( -375\.\d+\s+0\.\d+.* state 41')),
    ('stairs that rise past a box top are walked, not jumped', 'SPEC-0018 Box push, R-0242; build choice R-0244', 200, 1056, ['--start', '-1,373,29'], {'NEOPETS_TRACE_PUSH': '-8,352', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1'}, has(r'200 pos \(\s+-6\.\d+\s+3[23]\.')),
    ('a door into a room puts the camera level behind the hero', 'SPEC-0027 Placement after a load', 400, 1152, ['--start', '79.09,-47,-127.79'], {'NEOPETS_TRACE_EYE': '120', 'NEOPETS_TRACE_PUSH': '79,-70'}, has(r'EYE 120 pack 1152 .* polar \[9\.\d+, [-0-9.]+, 0\.(19|20)')),
    ('arriving in 1098 by the 1152 door puts the eye in front of the hero', 'SPEC-0027 Arrival through a scripted door, R-0243', 300, 1152, ['--start', '79.09,-47,-127.79'], {'NEOPETS_TRACE_EYE': '10', 'NEOPETS_TRACE_PUSH': '79,-140'}, has(r'EYE \d+ pack 1098 hero \S+ state \d+ facing 0\.00,-1\.00 eye [\d.]+,-12[345]\.\d+,-6[3-7]\.')),
    ('a closed lever gate stops the hero', 'SPEC-0028 Character versus moving placed objects; SPEC-0042 1171', 200, 1171, ['--start', '9.6,34,-0.61'], {'NEOPETS_TRACE_PUSH': '9.6,60', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1'}, has(r'200 pos \(\s+[\d.]+\s+-0\.\d+\s+42\.\d')),
    ('the lever opens the gate', 'SPEC-0042 1171 lever L5', 500, 1171, ['--start', '16,58,-1'], {'NEOPETS_TRACE_SQUARE': '2,4,6,8,10,12', 'NEOPETS_TRACE_PUSH': '9.6,30', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1'}, has(r'500 pos \(\s+[\d.]+\s+-0\.\d+\s+(29|30|31)\.')),
    ('the shell petpet hides when hit, takes a jumping hero on its top and launches him', 'SPEC-0047 Class 20 State 1; build choice R-0245', 520, 1056, ['--give', '99', '--start', '73.5,709,23'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_FACE': '2:76,709;90:76,709;159:76,709', 'NEOPETS_TRACE_ATTACK': '100,101', 'NEOPETS_TRACE_STICK': '2:0,0;160:0,0.2;185:0,0', 'NEOPETS_TRACE_CROSS': '162'}, has(r'class 20 creature \d+ launched the hero')),
    ('the shield lesson launcher fires at the hero', 'SPEC-0044 1056 shield lesson; SPEC-0006 Locator lookup', 100, 1056, ['--give', '106', '--give', '99', '--start', '-20.5,423.9,19.5', '--flag', '22', '--flag', '23', '--flag', '24', '--flag', '26', '--flag', '180', '--flag', '185', '--flag', '187', '--flag', '188'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_MOTE_AT': '50:1:160', 'NEOPETS_TRACE_WORDS': '1056:12', 'NEOPETS_TRACE_WORDS_EVERY': '1'}, has(r'WORDS tick 8\d pack 1056: \[1\] shots 1 at \[\[-2')),
    ('a fire shield reflects the lesson fireball and ends the lesson', 'SPEC-0029 Behavior 9 rule 2; SPEC-0044 1056 shield lesson; build choice R-0246', 200, 1056, ['--give', '106', '--give', '99', '--start', '-20.5,423.9,19.5', '--flag', '22', '--flag', '23', '--flag', '24', '--flag', '26', '--flag', '180', '--flag', '185', '--flag', '187', '--flag', '188'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_MOTE_AT': '50:1:160', 'NEOPETS_TRACE_FACE': '60:-54,421.5', 'NEOPETS_TRACE_GUARD': '62'}, has(r'flags on \[190\]')),
    ('the hero rides the 822 moving platform', 'SPEC-0018 riding (ground attach carry)', 400, 822, ['--flag', '621', '--start', '-206.7,2340.5,226'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_PLACE': '20:-207,2340,0,232'}, has(r'400 pos \(\s+-2\d\d\.\d+\s+2[23]\d\.\d+\s+\d+\.\d+\) state  1')),
    ('a named save state saves and loads the hero back', 'this build (save states)', 260, 800, ['--start', '480,2380,297', '--flag', '55'], {'NEOPETS_STATES_DIR': 'target/states-scenario', 'NEOPETS_TRACE_STATE': 'save:test@60'}, has(r'save state on tick 60: ready true Ok\("s\d+(-\d+)?"\) at \[489\.07')),
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
