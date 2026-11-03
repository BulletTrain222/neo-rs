"""Runs SPEC-0058's goal lists for an automated player in one continuous headless run and writes docs/STORY-CHECK.md.

The hero is teleported to each goal and does its action until its Done condition holds; scenes, level words, flags and
items carry on from goal to goal (crates/neopets-app/src/story_run.rs). So it tests the scripts' logic, not the walking,
and a fight is ended by the developer defeat. A failed goal is a lead (the fault may be this checker's): its Done flags
and items are then forced so the next goal can run. Usage: python scripts/story_check.py <repo> [first goal] [last goal]"""
import os, re, shutil, subprocess, sys
from collections import Counter

repo = os.path.abspath(sys.argv[1])
first = int(sys.argv[2]) if len(sys.argv) > 2 else 0
last = int(sys.argv[3]) if len(sys.argv) > 3 else 10**9
os.chdir(repo)
private = os.path.join(repo, 'private', 'story')
os.makedirs(private, exist_ok=True)
exe = os.path.join(private, 'story.exe')
shutil.copy('target/release/neopets-app.exe', exe)
spec = open('specs/SPEC-0058-main-story.md', encoding='utf-8').read()
TICKS = {'WAIT': 3000, 'FIGHT': 3000}
TUPLE = re.compile(r'\((-?\d+(?:\.\d+)?),\s*(-?\d+(?:\.\d+)?|\.|y),\s*(-?\d+(?:\.\d+)?)\)')

doors = {}
m = re.search(r'Town doors of pack 1023[^:]*: (.*)', spec)
if m:
    for k, pack, x, y, z in re.findall(r'(\d+) pack (\d+) \((-?[\d.]+), (-?[\d.]+), (-?[\d.]+)\)', m.group(1)):
        doors[int(k)] = (float(x), float(y), float(z))


def conditions(text):
    """Flags on, flags off, items held, items gone and a pack, from a Needs or Done cell. None when nothing parses."""
    t = re.sub(r'\([^)]*\)', ' ', text)
    on, off, held, gone, pack = set(), set(), set(), set(), None

    def numbers(s):
        out = set()
        for a, b in re.findall(r'(\d+) to (\d+)', s):
            out |= set(range(int(a), int(b) + 1))
        return out | {int(x) for x in re.findall(r'\d+', s)}
    for m in re.finditer(r'((?:\b\d{1,3}\b(?: to |, and |, | and )?)+) (on|off)\b', t):
        (on if m.group(2) == 'on' else off).update(n for n in numbers(m.group(1)) if n <= 742)
    for m in re.finditer(r'items? ((?:\d+(?:, | and )?)+) (held|gone)', t):
        (held if m.group(2) == 'held' else gone).update(numbers(m.group(1)))
    m = re.search(r'pack (\d{3,4}) current', t)
    if m:
        pack = int(m.group(1))
    if not (on or off or held or gone or pack):
        return None
    return dict(on=on, off=off, held=held, gone=gone, pack=pack)


goals = []
for act in ('Act 1', 'Act 2', 'Act 3', 'Act 4'):
    i = spec.index(f'## Goal list for an automated player ({act})')
    j = spec.find('\n## ', i + 10)
    for l in spec[i:j].splitlines():
        m = re.match(r'\| *(\d+) *\| *([A-Z]+) *\|(.*)\|\s*$', l)
        if m:
            c = [x.strip() for x in m.group(3).split('|')]
            goals.append(dict(act=act, n=int(m.group(1)), kind=m.group(2), packs=[int(x) for x in re.findall(r'\b\d{3,4}\b', c[0])], where=c[0], target=c[1], needs=conditions(c[2]), done=conditions(c[3]), raised=[int(x) for s in re.findall(r'\(([\d, ]+) turn on', c[3]) for x in re.findall(r'\d+', s)], done_text=c[3]))


def spots(g):
    """Where the hero stands for this goal: a town door's centre, or each point the row names."""
    d = re.search(r'[Tt]own door (\d+)', g['target'])
    if g['kind'] == 'DOOR' and d and int(d.group(1)) in doors:
        return [doors[int(d.group(1))]]
    if g['kind'] == 'TALK':
        return []
    ts = [(float(x), None if y in ('.', 'y') else float(y), float(z)) for x, y, z in TUPLE.findall(g['target'])]
    if g['kind'] == 'WALK':
        return ts[-1:]
    if g['kind'] == 'PICK':
        return ts
    if ts:
        return ts[:1]
    # "(as goal 16)", "(goal 52)": the point of the goal it names.
    back = re.search(r'goal (\d+)', g['target'])
    for o in goals:
        if back and o['n'] == int(back.group(1)) and o is not g:
            return spots(o)
    return []


def nums(s):
    return ','.join(str(x) for x in sorted(s))


lines = []
for g in goals:
    if not first <= g['n'] <= last:
        continue
    exits = g['done'] and g['done']['pack'] and not re.search(r'[Ll]egs?', g['target'])
    pack = (g['packs'][-1] if g['kind'] == 'WALK' and not exits else g['packs'][0]) if g['packs'] else ''
    if g['kind'] == 'WAIT':
        pack = ''
    name, ids = '', ''
    if g['kind'] == 'TALK':
        e = re.search(r'[Ee]ntit(?:y|ies) (\d+)', g['target'])
        if e:
            k = int(e.group(1))
            ids = nums(range(k, k + 8)) if re.search('famil|placements', g['target']) else str(k)
        name = re.split(r'[ (,.]', g['target'])[0]
        two = re.match(r'([A-Z][a-z]+ [A-Z][a-z]+)', g['target'])
        name = two.group(1) if two else name
    # Fights that need a real attack of a given kind, not a defeat: the checker cannot perform them yet.
    if g['kind'] == 'FIGHT' and re.search(r'[Ll]esson|Clear the five drains|Mote tasks|Knight record|^Record \d|lever|hit all', g['target']):
        g['kind'] = 'ATTACK'
    tr = re.search(r'[Tt]rigger(?: index)? (\d+)', g['target'])
    trigger = tr.group(1) if tr and g['kind'] in ('SQUARE', 'WALK', 'DOOR') else ''
    n, d = g['needs'] or dict(on=set(), off=set(), held=set()), g['done']
    done = []
    if d:
        done = [f'f{x}' for x in sorted(d['on'])] + [f'!f{x}' for x in sorted(d['off'])] + [f'i{x}' for x in sorted(d['held'])] + [f'!i{x}' for x in sorted(d['gone'])] + ([f"p{d['pack']}"] if d['pack'] else [])
    z = d or dict(on=set(), off=set(), held=set(), gone=set())
    place = ';'.join(f'{a},{c}' + (f',{b}' if b is not None else '') for a, b, c in spots(g))
    lines.append('\t'.join([str(g['n']), g['kind'], str(pack), place, name, ids, nums(n['on']), nums(n['off']), nums(n['held']), ','.join(done), nums(set(z['on']) | set(g['raised'])), nums(z['off']), nums(z['held']), nums(z['gone']), str(TICKS.get(g['kind'], 1500)), trigger]))
path = os.path.join(private, 'goals.tsv')
open(path, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines) + '\n')
env = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}
env.update(NEOPETS_TRACE_GOD='1', NEOPETS_TRACE_STAND='1', NEOPETS_TRACE_STORY=path)
r = subprocess.run([exe, '--play-trace', '5000000', '--pack', '909', '--seed', '1'], cwd=repo, env=env, capture_output=True, text=True, errors='replace', stdin=subprocess.DEVNULL, timeout=3000)
text = r.stdout + r.stderr
open(os.path.join(private, 'run.log'), 'w', encoding='utf-8').write(text)
status = {int(a): (b, int(c)) for a, b, c in re.findall(r'^GOAL (\d+) (\w+) tick \d+ pack (\d+)', text, re.M)}
needed = {int(a): b for a, b in re.findall(r'^NEEDED (\d+) (\S+)', text, re.M)}
panic = re.search(r'panicked at [^\n]*\n[^\n]*', text)
finished = 'STORY finished' in text
results = [(g, status.get(g['n'], ('notrun', 0))[0], needed.get(g['n'], '')) for g in goals if first <= g['n'] <= last]
counts = Counter(x[1] for x in results)
out = ['# Story check', '', "Written by `scripts/story_check.py` from SPEC-0058's goal lists, in one continuous headless run. The hero is teleported to each goal and fights end by the developer defeat, so this tests script logic, not walking, fighting or feel. A failed goal is a lead that may be the checker's own fault: its Done flags and items are forced so the next goal can run, and a failure can follow from one before it.", '']
out.append(f"The run {'reached the end of the list' if finished else 'did not reach the end'} (exit {r.returncode}).")
if panic:
    out.append('Crash: ' + panic.group(0)[:300])
out += ['', '| Result | Goals | Meaning |', '| --- | --- | --- |']
meaning = dict(notrun='Not reached', fail='The Done condition never held', gameover='The hero died and the world was torn down', manual='Kind not automated (minigames, pushes, climbs, shots)', nocheck='Ran, but its Done condition is not machine-readable', already='Done already held from earlier goals', **{'pass': 'Done condition reached'})
for k in meaning:
    out.append(f'| {k} | {counts[k]} | {meaning[k]} |')
for title, keep in (('Failed goals', ('fail',)), ('Hero died', ('gameover',)), ('Not reached', ('notrun',)), ('Not automated', ('manual',)), ('No readable Done condition', ('nocheck',))):
    out += ['', f'## {title}', '']
    for g, s, need in results:
        if s in keep:
            note = f' Needed and not met: {need}.' if need else ''
            out.append(f"- {g['act']} goal {g['n']} {g['kind']} ({g['where']}): {g['target'][:110]} Done: {g['done_text'][:70]}.{note}")
if first == 0 and last == 10**9:
    open('docs/STORY-CHECK.md', 'w', encoding='utf-8', newline='\n').write('\n'.join(out) + '\n')
print(dict(counts), 'finished' if finished else 'NOT finished', panic.group(0)[:200] if panic else '')
