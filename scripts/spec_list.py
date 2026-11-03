"""Writes docs/SPEC-CHECKLIST.md: everything the specs describe, as a progress list.

Every spec section, every level module with its spec lines, every walkthrough step, every automated-player goal and every
waiting request, each as a checkbox. A box is ticked by hand; a rerun keeps the ticks of items whose text has not changed.
The tags are evidence found by text matching, never a sign-off. Reads only specs/, crates/ and docs/. Usage:
python scripts/spec_list.py <repo>"""
import glob, os, re, sys

os.chdir(sys.argv[1] if len(sys.argv) > 1 else '.')
OUT = 'docs/SPEC-CHECKLIST.md'


def read(p):
    return open(p, encoding='utf-8', errors='replace').read()


def clean(s, n=150):
    s = re.sub(r'[`*|]', '', s).strip()
    s = re.sub(r'\s+', ' ', s)
    return s if len(s) <= n else s[:n - 3].rstrip() + '...'


ticked = set()
if os.path.exists(OUT):
    for l in read(OUT).splitlines():
        m = re.match(r'\s*- \[[xX]\] (.*)$', l)
        if m:
            ticked.add(re.sub(r' \{[^}]*\}$', '', m.group(1)))

code = ''
for f in glob.glob('crates/*/src/*.rs'):
    code += read(f).split('#[cfg(test)]')[0] + '\n'
status = read('docs/STATUS.md')
hay = (code + status).lower()
level = read('crates/neopets-sim/src/level.rs').split('#[cfg(test)]')[0] + read('crates/neopets-app/src/play.rs')
counts = []
body = []


def item(text, tag='', indent=0):
    """A checkbox line; returns True when it is ticked."""
    text = clean(text)
    done = text in ticked
    body.append('  ' * indent + f"- [{'x' if done else ' '}] {text}" + (f' {{{tag}}}' if tag else ''))
    return done


class Tally:
    def __init__(self, name):
        self.name, self.total, self.done, self.seen = name, 0, 0, 0

    def add(self, text, tag='', indent=0, seen=False):
        self.total += 1
        self.seen += bool(seen)
        self.done += item(text, tag, indent)


STOP = {'Summary', 'Layout', 'Behavior', 'Behaviour', 'Open questions', 'Edge cases and error handling', 'Test vectors', 'Where it lives on the disc'}


def mentioned(h):
    key = re.sub(r'\s*\(.*$', '', h).lower()
    key = re.sub(r'^[a-z]?\d+[a-z]?(\.\d+)*\.?\s*', '', key)
    beh = re.match(r'(behaviou?r \d+[a-z]?)', key)
    if beh:
        key = beh.group(1)
    return len(key) >= 5 and key in hay


# 1. Every spec, section by section.
body += ['## 1. Every spec, section by section', '']
level_specs = []
for f in sorted(glob.glob('specs/SPEC-*.md')):
    t = read(f)
    name = re.search(r'SPEC-(\d+)', f).group(1)
    title = clean(re.sub(r'^# SPEC-\d+:\s*', '', t.splitlines()[0]), 90)
    if 'level-scripts' in f:
        level_specs.append((f, name, title))
    tally = Tally(f'SPEC-{name} {title}')
    cited = len(re.findall('SPEC-' + name, code))
    body += [f'### SPEC-{name}: {title}', '', f'Cited {cited} times in the game code.', '']
    tally.add(f'SPEC-{name} {title}: the whole spec', f'{cited} code citations' if cited else 'no code citations', seen=cited > 0)
    for m in re.finditer(r'(?m)^(#{2,5}) (.+)$', t):
        h = clean(m.group(2), 120)
        if re.match(r'\d{3,4}\b', h) and 'level-scripts' in f:
            continue
        depth = len(m.group(1)) - 2
        seen = h not in STOP and mentioned(h)
        tally.add(f'SPEC-{name} {h}', 'named in code or notes' if seen else '', depth, seen)
    counts.append(tally)
    body.append('')

# 2. Every level module, with each line its spec gives.
body += ['## 2. Every level module', '', 'Each module is a world pack\'s script. Its sub-items are the spec\'s own lines for that module.', '']
tally = Tally('Level modules and their lines')
for f, name, _ in level_specs:
    cur, lines = None, []
    sections = []
    for line in read(f).splitlines():
        m = re.match(r'#{2,4} (\d{3,4})\b(.*)', line)
        if m:
            cur = (int(m.group(1)), clean(m.group(2).lstrip(': '), 90), [])
            sections.append(cur)
        elif line.startswith('## '):
            cur = None
        elif cur and re.match(r'\s*- ', line):
            cur[2].append(line)
    for p, what, rows in sections:
        has = bool(re.search(r'(?<![\d.])%d(?![\d.])' % p, level))
        tally.add(f'Pack {p} (SPEC-{name}): {what}', 'code found' if has else 'no code found', 0, has)
        for r in rows:
            d = (len(r) - len(r.lstrip())) // 2 + 1
            tally.add(f'Pack {p}: ' + re.sub(r'^\s*- ', '', r), '', min(d, 3))
counts.append(tally)
body.append('')

# 3. The story.
story = read('specs/SPEC-0058-main-story.md')
body += ['## 3. The main story, step by step (SPEC-0058)', '']
tally = Tally('Walkthrough steps')
walk = story[story.index('### Walkthrough'):story.index('### Endings')]
for m in re.finditer(r'(?m)^\| *(\d+) *\| *[TRB] *\| *([^|]*)\|[^|]*\|([^|]*)\|', walk):
    tally.add(f'Step {m.group(1)}, {clean(m.group(2), 50)}: {m.group(3)}')
counts.append(tally)
body.append('')

# 4. Every goal, with the story checker's last result.
results = {}
if os.path.exists('docs/STORY-CHECK.md'):
    sec = None
    for l in read('docs/STORY-CHECK.md').splitlines():
        if l.startswith('## '):
            sec = l[3:]
        m = re.match(r'- Act \d goal (\d+)', l)
        if m and sec:
            results[int(m.group(1))] = sec
body += ['## 4. Every automated-player goal (SPEC-0058)', '', 'Tags are the story checker\'s last result (docs/STORY-CHECK.md). It teleports to each goal, so a pass tests the script logic, not walking or fighting.', '']
for act in ('Act 1', 'Act 2', 'Act 3', 'Act 4'):
    tally = Tally(f'Goals, {act}')
    body += [f'### {act}', '']
    i = story.index(f'## Goal list for an automated player ({act})')
    j = story.find('\n## ', i + 10)
    for l in story[i:j].splitlines():
        m = re.match(r'\| *(\d+) *\| *([A-Z, ]+?) *\|([^|]*)\|([^|]*)\|', l)
        if m:
            n = int(m.group(1))
            r = results.get(n, 'passed or already done')
            tally.add(f'Goal {n} {m.group(2)} ({clean(m.group(3), 30)}): {m.group(4)}', r, 0, r == 'passed or already done')
    counts.append(tally)
    body.append('')

# 5. Questions still waiting.
body += ['## 5. Questions waiting on the research side (specs/REQUESTS.md)', '']
tally = Tally('Requests waiting')
for l in read('specs/REQUESTS.md').splitlines():
    if l.startswith('| R-'):
        c = [x.strip() for x in l.strip().strip('|').split(' | ')]
        if not c[-1].lower().startswith('answered') and c[0] != 'R-0000':
            tally.add(f'{c[0]}: {c[1]}', '' if c[-1].startswith('Open') else 'no status recorded')
counts.append(tally)
body.append('')

# 6. What the code itself says is not built.
body += ['## 6. Things the code says are not built', '']
tally = Tally('Stated gaps in the code')
for f in sorted(glob.glob('crates/*/src/*.rs')):
    t = read(f).split('#[cfg(test)]')[0]
    for l in t.splitlines():
        if re.search(r'//.*\b(not built|Not built|not implemented|Not implemented|not reproduced|Not reproduced|TODO|FIXME)\b', l):
            tally.add(f'{os.path.basename(f)}: ' + l.strip().lstrip('/!').strip())
counts.append(tally)

total = sum(c.total for c in counts)
done = sum(c.done for c in counts)
head = ['# Spec checklist', '', 'Everything the specs describe, generated by `scripts/spec_list.py`. Tick a box by changing `[ ]` to `[x]`; a rerun keeps your ticks as long as the item\'s text is unchanged. Tags in `{ }` are evidence from text matching, not a sign-off: "named in code or notes" means the section\'s title appears there, and many built sections use other wording, so a section without the tag may still be built.', '', 'This covers what the spec files state, down to their sections and level-script lines. Numbers inside a spec\'s tables are part of their section, not separate items.', '', f'**Ticked: {done} of {total}.**', '', '| Part | Items | Ticked | Evidence found |', '| --- | --- | --- | --- |']
for c in counts:
    head.append(f'| {c.name} | {c.total} | {c.done} | {c.seen if c.seen else "-"} |')
open(OUT, 'w', encoding='utf-8', newline='\n').write('\n'.join(head + [''] + body) + '\n')
print(total, 'items,', done, 'ticked,', len(counts), 'parts')
