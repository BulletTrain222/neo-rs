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


def one_return_to_1098(out):
    entered = len(set(re.findall(r'entered interior 1156 at entry record \d+ on tick (\d+)', out)))
    back = re.search(r'hero moved to locator 3 of 1098', out)
    if not back:
        return 'the hero never left 1156'
    return None if entered == 1 else f'entered 1156 {entered} times (the door at locator 3 sent him back)'


def sisters_follow(out):
    last = re.findall(r'SISTERS tick 1650: (.*)', out)
    if not last:
        return 'no sister line at tick 1650'
    zs = [float(z) for z in re.findall(r'\(-?\d+, -?\d+, (-?\d+)\)', last[0])]
    if len(zs) != 3:
        return f'not three sisters: {last[0]}'
    return None if all(z < 52 for z in zs) else f'a sister stayed in section 0: {last[0]}'


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


def both(a, b):
    def test(out):
        return a(out) or b(out)
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
    ('800 basin ladder on object 0 climbs after the finale', 'SPEC-0028 7.2', 320, 800, ['--start', '472.33,2366.11,297', '--flag', '55'], {'NEOPETS_TRACE_TOWARD': '1:469.65,2361.9', 'NEOPETS_TRACE_STICK': '20:1,0', 'NEOPETS_TRACE_CLIMB': '1'}, has(r'300 pos \(\s+4[67]\d\.\d+\s+305\.')),
    ('a ladder mounts from the floor and climbs', 'SPEC-0033 Behavior 13', 220, 1171, ['--start', '-374.98,3.12,-9.79'], {'NEOPETS_TRACE_CLIMB': '1'}, has(r'200 pos \( -375\.\d+\s+0\.\d+.* state 41')),
    ('stairs that rise past a box top are walked, not jumped', 'SPEC-0018 Box push, R-0242; build choice R-0244', 200, 1056, ['--start', '-1,373,29'], {'NEOPETS_TRACE_PUSH': '-8,352', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1'}, has(r'200 pos \(\s+-6\.\d+\s+3[23]\.')),
    ('a door into a room puts the camera level behind the hero', 'SPEC-0027 Placement after a load', 400, 1152, ['--start', '79.09,-47,-127.79'], {'NEOPETS_TRACE_EYE': '120', 'NEOPETS_TRACE_PUSH': '79,-70'}, has(r'EYE 120 pack 1152 .* polar \[9\.\d+, [-0-9.]+, 0\.(19|20)')),
    ('a plain 1098 arrival camera is clipped inside the hall and held there while the hero stands', 'SPEC-0027 Occlusion clip, Eye frozen against a remembered wall; R-0256', 200, 1098, [], {'NEOPETS_TRACE_EYE': '10', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1'}, both(has(r'EYE 0 pack 1098 hero \S+ state \d+ facing 0\.00,1\.00 eye [-\d.]+,-12\d\.\d+,-10[0-3]\.'), has(r'EYE 190 pack 1098 hero \S+ state \d+ facing 0\.00,1\.00 eye [-\d.]+,-12\d\.\d+,-10[0-3]\.'))),
    ('the 956 lift carries the hero from the bottom to the top', 'SPEC-0043 956 Lift, R-0259', 420, 956, ['--flag', '211', '--flag', '208', '--flag', '209', '--flag', '210', '--flag', '623', '--start', '-703.7,-825,8'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1'}, has(r'400 pos \(\s+-703\.\d+\s+6[6-9]\.\d+\s+-824\.\d+\) state  1 phase  0 action\s+\d+ gap\s+0\.0')),
    ('a jump after the 956 lift ride comes back down', 'SPEC-0043 956 Lift, R-0259', 560, 956, ['--flag', '211', '--flag', '208', '--flag', '209', '--flag', '210', '--flag', '623', '--start', '-703.7,-825,8'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_CROSS': '450'}, has(r'559 pos \(\s+-703\.\d+\s+6[6-9]\.\d+\s+-824\.\d+\) state  1 ')),
    ('the 1174 stump of fallen tile 43 is caught from a standing jump and climbed', 'SPEC-0044 1174 Climbing the fallen pillar, SPEC-0033 13b, R-0263', 360, 1174, ['--start', '12.4,17.5,-128.1'], {'NEOPETS_DEV_FIRE': '5:43', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_PUSH': '0,17.5', 'NEOPETS_TRACE_CROSS': '255'}, both(has(r'26[3-9] pos \(\s+11\.7\d+\s+-126\.68\d\s+17\.\d+\) state 30'), has(r'35\d pos \(\s+[6-8]\.\d+\s+-12[12]\.\d+\s+1[78]\.\d+\) state  3'))),
    ('all three 1174 sisters follow into section 1 after its checkpoint', 'SPEC-0011 Movement goals, R-0262', 1660, 1174, ['--start', '0,40,-127'], {'NEOPETS_TRACE_SISTERS': '1', 'NEOPETS_TRACE_PUSH': '0,75', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1'}, sisters_follow),
    ('leaving 1156 lands at 1098 locator 3 and stays out while standing in the door zone', 'SPEC-0006 Doors, R-0257', 700, 1098, ['--start', '88.1,-61,-121'], {'NEOPETS_TRACE_PUSH': '88,-75', 'NEOPETS_TRACE_PUSH_PACK': '1098', 'NEOPETS_TRACE_TOWARD': '250:89,-60', 'NEOPETS_TRACE_STICK': '250:-;330:0,0'}, one_return_to_1098),
    ('arriving in 1098 by the 1152 door puts the eye in front of the hero', 'SPEC-0027 Arrival through a scripted door, R-0243', 300, 1152, ['--start', '79.09,-47,-127.79'], {'NEOPETS_TRACE_EYE': '10', 'NEOPETS_TRACE_PUSH': '79,-140'}, has(r'EYE \d+ pack 1098 hero \S+ state \d+ facing 0\.00,-1\.00 eye [\d.]+,-12[345]\.\d+,-6[3-7]\.')),
    ('a closed lever gate stops the hero', 'SPEC-0028 Character versus moving placed objects; SPEC-0042 1171', 200, 1171, ['--start', '9.6,34,-0.61'], {'NEOPETS_TRACE_PUSH': '9.6,60', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1'}, has(r'200 pos \(\s+[\d.]+\s+-0\.\d+\s+42\.\d')),
    ('the lever opens the gate', 'SPEC-0042 1171 lever L5', 500, 1171, ['--start', '16,58,-1'], {'NEOPETS_TRACE_SQUARE': '2,4,6,8,10,12', 'NEOPETS_TRACE_PUSH': '9.6,30', 'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1'}, has(r'500 pos \(\s+[\d.]+\s+-0\.\d+\s+(29|30|31)\.')),
    ('the 800 Chieftain walks off the rim into the basin after stage B', 'SPEC-0014 phase 1, SPEC-0041 800 stage B, R-0253', 700, 800, ['--start', '480,2380,297'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_DEV_WORD': '5:1:1', 'NEOPETS_DEV_KILL_ENTITY': '10:10,11', 'NEOPETS_TRACE_CHARS': '60'}, has(r'Ixi Chieftan \[class 11 hp 300/300 ixi \w+(\([^)]*\))? at 5\d\d,29[5-7]\.\d,')),
    ('a 1091 wall button moves the pillars to the stage 1 layout', 'SPEC-0044 1091 Stage puzzle, R-0269', 420, 1091, [], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_DEV_FIRE': '60:68:1', 'NEOPETS_TRACE_WORDS': '1091:100,101,110'}, has(r'WORDS tick 360 pack 1091: \[1, 0, 8333\]')),
    ('a pack 816 breakable blast breaks its neighbour 5.2 away on the 20th tick', 'SPEC-0048 Break routes, blast chain', 60, 816, [], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_SMASH': '20:0', 'NEOPETS_TRACE_BREAK': '1'}, has(r'breakable 1 of 816 broken at .* on tick 40 ')),
    ('the shell petpet hides when hit and launches a hero jumping at it', 'SPEC-0047 Class 20 State 1, SPEC-0040 the character push rule, R-0245', 540, 1056, ['--give', '99', '--start', '73.5,709,23'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_FACE': '2:76,709;90:76,709;159:76,709', 'NEOPETS_TRACE_ATTACK': '100,101', 'NEOPETS_TRACE_PUSH': '76,709', 'NEOPETS_TRACE_CROSS': '162,215,268,321,374,427,480'}, has(r'class 20 creature \d+ launched the hero')),
    ('the shield lesson launcher fires at the hero', 'SPEC-0044 1056 shield lesson; SPEC-0006 Locator lookup', 100, 1056, ['--give', '106', '--give', '99', '--start', '-20.5,423.9,19.5', '--flag', '22', '--flag', '23', '--flag', '24', '--flag', '26', '--flag', '180', '--flag', '185', '--flag', '187', '--flag', '188'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_MOTE_AT': '50:1:160', 'NEOPETS_TRACE_WORDS': '1056:12', 'NEOPETS_TRACE_WORDS_EVERY': '1'}, has(r'WORDS tick 8\d pack 1056: \[1\] shots 1 at \[\[-2')),
    ('a fire shield reflects the lesson fireball and ends the lesson', 'SPEC-0029 Behavior 9 rule 2; SPEC-0044 1056 shield lesson; build choice R-0246', 200, 1056, ['--give', '106', '--give', '99', '--start', '-20.5,423.9,19.5', '--flag', '22', '--flag', '23', '--flag', '24', '--flag', '26', '--flag', '180', '--flag', '185', '--flag', '187', '--flag', '188'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_MOTE_AT': '50:1:160', 'NEOPETS_TRACE_FACE': '60:-54,421.5', 'NEOPETS_TRACE_GUARD': '62'}, has(r'flags on \[190\]')),
    ('the hero rides the 822 moving platform', 'SPEC-0018 riding (ground attach carry)', 400, 822, ['--flag', '621', '--start', '-206.7,2340.5,226'], {'NEOPETS_TRACE_NOAUTO': '1', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1', 'NEOPETS_TRACE_PLACE': '20:-207,2340,0,232'}, has(r'400 pos \(\s+-2\d\d\.\d+\s+2[23]\d\.\d+\s+\d+\.\d+\) state  1')),
    ('a named save state saves and loads the hero back', 'this build (save states)', 260, 800, ['--start', '480,2380,297', '--flag', '55'], {'NEOPETS_STATES_DIR': 'target/states-scenario', 'NEOPETS_TRACE_STATE': 'save:test@60'}, has(r'save state on tick 60: ready true Ok\("s\d+(-\d+)?"\) at \[489\.07')),
    ('sound records match the spec', 'SPEC-0031 types 4 and 5', 1, 909, [], {'NEOPETS_TRACE_CUE_RECORDS': '1'}, has(r'CUE RECORDS type 4 81, type 5 24')),
    # Co-op (docs/MULTIPLAYER.md; this build's feature, not the original's).
    ('co-op: player 2 goes down and is revived beside player 1', 'this build (co-op rules)', 160, 909, [], {'NEOPETS_COOP': '1', 'NEOPETS_COOP_STILL': '1', 'NEOPETS_COOP_HP': '0', 'NEOPETS_TRACE_STAND': '1', 'NEOPETS_TRACE_GOD': '1'}, both(has(r'COOP 30 p2 .* down Some'), has(r'COOP 150 p2 .* hp [1-9]\d* down None'))),
    ('co-op: player 2 drinks from its own quick panel', 'this build (co-op panels)', 200, 909, [], {'NEOPETS_COOP': '1', 'NEOPETS_COOP_STILL': '1', 'NEOPETS_COOP_HP': '40', 'NEOPETS_COOP_DRINK': '1', 'NEOPETS_TRACE_GIVE': '2:140:1,2:141:1,2:142:1,2:143:1,2:144:1,2:145:1,2:146:1,2:147:1,2:148:1,2:149:1,2:150:1,2:151:1,2:152:1,2:153:1,2:154:1,2:155:1,2:156:1,2:157:1,2:158:1,2:159:1'}, has(r'used item 146 on tick \d+: hp 120')),
    ('co-op: player 2 takes both players into the barn and out', 'this build (co-op doors)', 330, 909, [], {'NEOPETS_COOP': '1', 'NEOPETS_COOP_STILL': '1', 'NEOPETS_COOP_DOOR': '0@60;0@250', 'NEOPETS_TRACE_STAND': '1'}, both(has(r'COOP 70 door: world 909 .*Some\(13\)'), both(has(r'COOP 2[0-4]0 door: world 951'), has(r'COOP (2[6-9]|3[0-2])0 door: world 909')))),
    ('co-op: player 2 moves talk boxes on', 'this build (co-op talk boxes)', 60, 909, [], {'NEOPETS_COOP': '1', 'NEOPETS_COOP_STILL': '1', 'NEOPETS_COOP_BOX': '1', 'NEOPETS_TRACE_COOPBOX': '1'}, has(r'COOPBOX tick \d+: .*dialogue false')),
    ('co-op: player 2 mounts the 1171 ladder and climbs', 'this build (co-op climbing; SPEC-0033 Behavior 13)', 200, 1171, ['--start', '-370,3.12,-9.79'], {'NEOPETS_COOP': '1', 'NEOPETS_COOP_CLIMB': '1', 'NEOPETS_COOP_AT': '-374.98,3.12,-9.79@100', 'NEOPETS_TRACE_STAND': '1'}, both(has(r'COOP 110 p2 at .* state 41 ladder true'), has(r'COOP 180 p2 at \(-374\.98, -[0-7]\.\d+, 3\.12\) state 41'))),
    ('co-op: a story switch makes the partner the other hero', 'this build (co-op hero switch; SPEC-0053 3.2)', 200, 909, [], {'NEOPETS_COOP': '1', 'NEOPETS_COOP_STILL': '1', 'NEOPETS_COOP_SWITCH': '100'}, has(r'COOP 180 heroes: player 1 hero 1, player 2 hero Some\(0\) hp Some\(\(150, 150\)\)')),
    ('co-op: a second Tor guards with a shield', 'this build (co-op guard)', 100, 909, [], {'NEOPETS_COOP': '1', 'NEOPETS_COOP_TOR': '1', 'NEOPETS_COOP_GUARD': '1', 'NEOPETS_TRACE_GIVE': '2:106:1'}, has(r'COOP 90 p2 .* state 20 clip 63')),
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
