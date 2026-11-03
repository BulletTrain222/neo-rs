"""What is still missing: compares specs/ with this folder's code and notes, and writes docs/MISSING.md.

Reads only specs/, crates/ and docs/STATUS.md. It needs no disc. Every list is a set of leads found by text matching, not
proof: a lead may be built under another wording, and a level with no leads may still play wrongly. Usage:
python scripts/missing.py <repo>"""
import glob, os, re, sys
from collections import defaultdict

repo = sys.argv[1] if len(sys.argv) > 1 else '.'
os.chdir(repo)


def read(p):
    return open(p, encoding='utf-8', errors='replace').read()


game = {}
for f in glob.glob('crates/*/src/*.rs'):
    game[f.replace('\\', '/')] = read(f).split('#[cfg(test)]')[0]
level = game['crates/neopets-sim/src/level.rs']
play = game['crates/neopets-app/src/play.rs']
code = '\n'.join(game.values())
status = read('docs/STATUS.md')
out = []


def w(s=''):
    out.append(s)


# 1. Level modules: each handler a level spec names, against the match arms for that pack.
arms = defaultdict(set)
packs_in_code = set()
for m in re.finditer(r'\((\d{3,4}),\s*([0-9 |.=]+?)\)\s*(?:if\b[^\n]*?)?=>', level + play):
    p = int(m.group(1))
    for a, b in re.findall(r'(\d+)\.\.=(\d+)', m.group(2)):
        arms[p] |= set(range(int(a), int(b) + 1))
    arms[p] |= {int(x) for x in re.findall(r'\d+', m.group(2))}
for m in re.finditer(r'(?m)^\s*((?:\d{3,4}\s*\|\s*)*\d{3,4})\s*(?:if\b[^\n]*?)?=>', level):
    packs_in_code |= {int(x) for x in re.findall(r'\d+', m.group(1))}
packs_in_code |= set(arms)
sections = {}
for f in sorted(glob.glob('specs/SPEC-00*level-scripts*.md')):
    name = re.search(r'SPEC-(\d+)', f).group(1)
    cur = None
    for line in read(f).splitlines():
        m = re.match(r'#{2,4} (\d{3,4})\b', line)
        if m:
            cur = int(m.group(1))
            sections.setdefault(cur, [name, []])
        elif line.startswith('## '):
            cur = None
        elif cur:
            sections[cur][1].append(line)
no_code, handler_gaps = [], []
for p, (spec, lines) in sorted(sections.items()):
    text = '\n'.join(lines)
    if p not in packs_in_code and not re.search(r'\b%d\b' % p, level):
        if not re.search(r'do nothing|no-op|: nothing|stores the world handle', text):
            no_code.append((p, spec, len(lines)))
        continue
    hs = {int(x) for x in re.findall(r'[Hh]andler (\d+)', text)}
    gone = sorted(h for h in hs if h not in arms[p])
    if gone and arms[p]:
        handler_gaps.append((p, spec, gone, sorted(hs)))

# 2. Stated gaps: the comments in the code that say something is not built.
stated = []
for f, t in game.items():
    for i, l in enumerate(t.splitlines(), 1):
        if re.search(r'//.*\b(not built|Not built|not implemented|Not implemented|not reproduced|Not reproduced|TODO|FIXME)\b', l):
            stated.append((os.path.basename(f), i, l.strip().lstrip('/!').strip()))

# 3. How often the code and the notes cite each spec.
cites = []
for f in sorted(glob.glob('specs/SPEC-*.md')):
    name = re.search(r'SPEC-(\d+)', f).group(1)
    cites.append((len(re.findall('SPEC-' + name, code)), len(re.findall('SPEC-' + name, status)), name, os.path.basename(f)[10:-3]))

# 4. The story: walkthrough steps, and whether a status note says the step was traced.
story = read('specs/SPEC-0058-main-story.md')
walk = story[story.index('### Walkthrough'):story.index('### Endings')]
steps = [(int(m.group(1)), m.group(2).strip(), m.group(3).strip()[:90]) for m in re.finditer(r'(?m)^\| *(\d+) *\| *[TRB] *\| *([^|]*)\|[^|]*\|([^|]*)\|', walk)]
told = set()
for m in re.finditer(r'SPEC-0058[^.;)*]*?steps? ([0-9, and to]+)', status):
    s = m.group(1)
    for a_, b_ in re.findall(r'(\d+) to (\d+)', s):
        told |= set(range(int(a_), int(b_) + 1))
    told |= {int(x) for x in re.findall(r'\d+', s)}
story_gap = [s for s in steps if s[0] not in told]
goals = []
for act in ('Act 1', 'Act 2', 'Act 3', 'Act 4'):
    i = story.index(f'## Goal list for an automated player ({act})')
    j = story.find('\n## ', i + 10)
    goals.append((act, len(re.findall(r'(?m)^\| *\d+ *\| *[A-Z, ]+? *\|', story[i:j]))))

# 5. Open requests, and the older status notes' words.
opens = []
for l in read('specs/REQUESTS.md').splitlines():
    if l.startswith('| R-'):
        c = [x.strip() for x in l.strip().strip('|').split(' | ')]
        if not c[-1].lower().startswith('answered') and c[0] != 'R-0000':
            opens.append((c[0], c[1] + ('' if c[-1].startswith('Open') else ' (no status recorded in its row)')))
untested = re.findall(r'[^.]*\bnot played\b[^.]*\.', status)

w('# What is still missing')
w()
w('Written by `scripts/missing.py` from the specs, the code and docs/STATUS.md, with no disc. These are leads from text matching, not proof: an item may be built under other wording, and a level with no leads may still play wrongly. Nothing here says the game has been completed by hand, because no run has.')
w()
w('| Check | Count |')
w('| --- | --- |')
w(f'| Level modules in the specs | {len(sections)} |')
w(f'| Modules with no code found | {len(no_code)} |')
w(f'| Modules with a spec handler that has no code arm | {len(handler_gaps)} |')
w(f'| Comments in the code naming something not built | {len(stated)} |')
w(f'| Specs no game code cites | {sum(1 for c in cites if c[0] == 0)} of {len(cites)} |')
w(f'| Walkthrough steps with no status note saying they were traced | {len(story_gap)} of {len(steps)} |')
w(f'| Automated-player goals in the spec, none run end to end | {sum(g[1] for g in goals)} ({", ".join(f"{a}: {n}" for a, n in goals)}) |')
w(f'| Requests waiting for an answer | {len(opens)} |')
w()
w('## Requests waiting for an answer')
w()
for r, t in opens:
    w(f'- {r}: {t}')
w()
w('## Walkthrough steps with no status note (SPEC-0058)')
w()
for n, pack, task in story_gap:
    w(f'- Step {n}, {pack}: {task}')
w()
w('## Level modules with no code found')
w()
for p, s, n in no_code:
    w(f'- {p} (SPEC-{s}, {n} spec lines)')
w()
w('## Level modules with spec handlers and no code arm')
w()
w('Weak leads. Many handlers are built as fixed rules or tables with no arm of that number (984, 1432, 1344, 1140 and 1133 were read by hand and do have code), so read the section before building.')
w()
for p, s, gone, hs in handler_gaps:
    w(f'- {p} (SPEC-{s}): handlers {gone} of {hs}')
w()
w('## Citations of each spec (game code, status notes), fewest first')
w()
for c_, s_, n, name in sorted(cites)[:25]:
    w(f'- SPEC-{n} {name}: {c_} in code, {s_} in notes')
w()
w('## Comments in the code naming something not built')
w()
for f, i, l in stated:
    w(f'- {f}:{i}: {l[:170]}')
open('docs/MISSING.md', 'w', encoding='utf-8', newline='\n').write('\n'.join(out) + '\n')
print('\n'.join(out[:14]))
