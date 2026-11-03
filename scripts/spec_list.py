"""Writes docs/SPEC-CHECKLIST.md: everything the specs describe, as a progress list.

Every spec section, table row and bullet; every level module with its spec lines; every walkthrough step and automated-
player goal; every request; every not-built comment. A rule ticks an item when it finds the evidence it names, and the
tag says which rule, so an automatic tick means "evidence found", never "played and confirmed". A box ticked by hand is
kept across reruns while the item's text is unchanged. Reads only specs/, crates/ and docs/. Usage:
python scripts/spec_list.py <repo>"""
import glob, os, re, sys
from collections import Counter

os.chdir(sys.argv[1] if len(sys.argv) > 1 else '.')
OUT = 'docs/SPEC-CHECKLIST.md'


def read(p):
    return open(p, encoding='utf-8', errors='replace').read()


def clean(s, n=170):
    s = re.sub(r'[`*|]', ' ', s).strip()
    s = re.sub(r'\s+', ' ', s)
    return s if len(s) <= n else s[:n - 3].rstrip() + '...'


# Hand ticks: a ticked line no rule ticked.
hand = set()
if os.path.exists(OUT):
    for l in read(OUT).splitlines():
        m = re.match(r'\s*- \[[xX]\] (.*)$', l)
        if m and '{auto:' not in l:
            hand.add(re.sub(r' \{[^}]*\}$', '', m.group(1)))

sources = {f.replace('\\', '/'): read(f).split('#[cfg(test)]')[0] for f in glob.glob('crates/*/src/*.rs')}
code = '\n'.join(sources.values())
status = read('docs/STATUS.md')
code_lower = code.lower()
cite_lines = {}
for l in code_lower.splitlines():
    for n in re.findall(r'spec-(\d{4})', l):
        cite_lines.setdefault(n, []).append(l)
addresses = {int(h.replace('_', ''), 16) for h in re.findall(r'0x([0-9A-Fa-f_]{5,10})', code)}
tally_rules = Counter()
counts = []
body = []


class Tally:
    def __init__(self, name):
        self.name, self.total, self.auto, self.hand = name, 0, 0, 0

    def add(self, text, rule='', note='', indent=0):
        """rule: why a rule ticks it; note: a tag that ticks nothing."""
        text = clean(text)
        self.total += 1
        by_hand = text in hand
        self.hand += by_hand
        self.auto += bool(rule) and not by_hand
        if rule and not by_hand:
            tally_rules[rule] += 1
        tag = f' {{auto: {rule}}}' if rule and not by_hand else (f' {{{note}}}' if note else '')
        body.append('  ' * min(indent, 5) + f"- [{'x' if rule or by_hand else ' '}] {text}{tag}")


STOP = {'Summary', 'Layout', 'Behavior', 'Behaviour', 'Open questions', 'Edge cases and error handling', 'Test vectors', 'Where it lives on the disc'}


def named_in_code(h, name):
    """A comment naming this spec also names the section's title, without its numbering and confidence note."""
    key = re.sub(r'\s*\(.*$', '', h).lower()
    key = re.sub(r'^[a-z]?\d+[a-z]?(\.\d+)*\.?\s*', '', key)
    beh = re.match(r'(behaviou?r \d+[a-z]?)', key)
    if beh:
        key = beh.group(1)
    return len(key) >= 8 and any(key in l for l in cite_lines.get(name, ()))


def address_rule(text):
    """Every boot-data address the line names (0x2xxxxx or 0x3xxxxx) is read in the code; at least one is named."""
    found = [int(h.replace('_', ''), 16) for h in re.findall(r'0x([0-9A-Fa-f_]{6,8})\b', text)]
    found = [a for a in found if 0x200000 <= a < 0x400000]
    return bool(found) and all(a in addresses for a in found)


# 1. Every spec: sections, table rows and bullets.
body += ['## 1. Every spec, in full', '']
level_specs = []
for f in sorted(glob.glob('specs/SPEC-*.md')):
    t = read(f)
    name = re.search(r'SPEC-(\d+)', f).group(1)
    title = clean(re.sub(r'^# SPEC-\d+:\s*', '', t.splitlines()[0]), 90)
    is_level = 'level-scripts' in f
    if is_level:
        level_specs.append((f, name))
    tally = Tally(f'SPEC-{name} {title}')
    cited = len(re.findall('SPEC-' + name, code))
    body += [f'### SPEC-{name}: {title}', '', f'Cited {cited} times in the game code.', '']
    tally.add(f'SPEC-{name} {title}: the code cites this spec', 'cited in code' if cited else '')
    lines = t.splitlines()
    depth, section, skipping = 0, '', False
    for k, line in enumerate(lines[1:], 1):
        h = re.match(r'(#{2,5}) (.+)$', line)
        if h:
            section = clean(h.group(2), 60)
            depth = len(h.group(1)) - 2
            skipping = is_level and bool(re.match(r'\d{3,4}\b', section))
            if skipping:
                continue
            tally.add(f'SPEC-{name} {clean(h.group(2), 120)}', 'title named in code' if section not in STOP and named_in_code(section, name) else '', indent=depth)
            continue
        if skipping or line.startswith('- Answers:') or line.startswith('- Status:') or line.startswith('- Disc build:') or line.startswith('- Confidence:'):
            continue
        row = line.startswith('|') and not re.match(r'\|[\s:|-]+\|?\s*$', line)
        if row and k + 1 < len(lines) and re.match(r'\|[\s:|-]+\|?\s*$', lines[k + 1]):
            continue  # a table's header
        bullet = re.match(r'\s*(?:[-*]|\d+\.)\s+(.+)', line)
        if not (row or bullet):
            continue
        text = line if row else bullet.group(1)
        tally.add(f'SPEC-{name}, {section}: {text}', 'address in code' if address_rule(text) else '', indent=depth + 1)
    counts.append(tally)
    body.append('')

# 2. Every level module, with each line its spec gives.
level = sources['crates/neopets-sim/src/level.rs'] + '\n' + sources['crates/neopets-app/src/play.rs']
src_lines = level.splitlines()


def pack_blocks(p):
    """The code of a pack: each match arm that names it, and what follows a comment that names it with its spec."""
    out = []
    arm = re.compile(r'^(\s*)(?:\(%d,|(?:\d{3,4} \| )*%d(?: \| \d{3,4})*\s*(?:if\b.*)?=>)' % (p, p))
    note = re.compile(r'SPEC-\d{4} %d\b' % p)
    for i, l in enumerate(src_lines):
        m = arm.match(l)
        if m:
            ind = len(m.group(1))
            j = i + 1
            while j < len(src_lines) and j < i + 400 and (not src_lines[j].strip() or len(src_lines[j]) - len(src_lines[j].lstrip()) > ind or src_lines[j].strip().startswith('}')):
                j += 1
                if src_lines[j - 1].strip().startswith('}') and len(src_lines[j - 1]) - len(src_lines[j - 1].lstrip()) == ind:
                    break
            out.append('\n'.join(src_lines[i:j]))
        elif note.search(l):
            out.append('\n'.join(src_lines[i:i + 40]))
    return '\n'.join(out)


NAMED = re.compile(r'(?:[Ff]lags?|SetGameState\(|game states?|LevelMsg\(|[Ll]ines?|[Mm]ovies?|PlayFMV\(|items?|GiveItemToHero\(item|[Ll]ocators?|[Cc]ameras?)\s*(\d{1,4})')


def numbers_rule(text, block):
    """Every flag, line, movie, item, locator and camera number the line names appears in this pack's code; at least two."""
    names = NAMED.findall(text)
    if len(names) < 2:
        return False
    return all(re.search(r'(?<![\d.])%s(?![\d.])' % n, block) for n in names)


body += ['## 2. Every level module', '']
tally = Tally('Level modules and their lines')
for f, name in level_specs:
    cur = None
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
        block = pack_blocks(p)
        text = '\n'.join(rows)
        idle = not block and re.search(r'do nothing|no-op|: nothing|stores the world handle', text)
        tally.add(f'Pack {p} (SPEC-{name}): {what}', 'module does nothing' if idle else ('code found' if block else ''))
        for r in rows:
            d = (len(r) - len(r.lstrip())) // 2 + 1
            line = re.sub(r'^\s*- ', '', r)
            if line.startswith(('Confidence', 'Family', 'Coverage')):
                continue
            tally.add(f'Pack {p}: {line}', 'numbers match the pack code' if numbers_rule(line, block) else '', indent=min(d, 3))
counts.append(tally)
body.append('')

# 3. The story steps: a status note that names the step.
story = read('specs/SPEC-0058-main-story.md')
told = set()
for m in re.finditer(r'SPEC-0058[^.;)*]*?steps? ([0-9, and to]+)', status):
    for a, b in re.findall(r'(\d+) to (\d+)', m.group(1)):
        told |= set(range(int(a), int(b) + 1))
    told |= {int(x) for x in re.findall(r'\d+', m.group(1))}
body += ['## 3. The main story, step by step (SPEC-0058)', '']
tally = Tally('Walkthrough steps')
walk = story[story.index('### Walkthrough'):story.index('### Endings')]
for m in re.finditer(r'(?m)^\| *(\d+) *\| *[TRB] *\| *([^|]*)\|[^|]*\|([^|]*)\|', walk):
    tally.add(f'Step {m.group(1)}, {clean(m.group(2), 50)}: {m.group(3)}', 'a status note traces it' if int(m.group(1)) in told else '')
counts.append(tally)
body.append('')

# 4. Every goal: a pass in the story check.
results = {}
if os.path.exists('docs/STORY-CHECK.md'):
    sec = None
    for l in read('docs/STORY-CHECK.md').splitlines():
        if l.startswith('## '):
            sec = l[3:]
        m = re.match(r'- Act \d goal (\d+)', l)
        if m and sec:
            results[int(m.group(1))] = sec
body += ['## 4. Every automated-player goal (SPEC-0058)', '']
for act in ('Act 1', 'Act 2', 'Act 3', 'Act 4'):
    tally = Tally(f'Goals, {act}')
    body += [f'### {act}', '']
    i = story.index(f'## Goal list for an automated player ({act})')
    j = story.find('\n## ', i + 10)
    for l in story[i:j].splitlines():
        m = re.match(r'\| *(\d+) *\| *([A-Z, ]+?) *\|([^|]*)\|([^|]*)\|', l)
        if m:
            n = int(m.group(1))
            r = results.get(n)
            tally.add(f'Goal {n} {m.group(2)} ({clean(m.group(3), 30)}): {m.group(4)}', '' if r else 'story check passed by teleport', note=f'story check: {r}' if r else '')
    counts.append(tally)
    body.append('')

# 5. Requests.
body += ['## 5. Every request to the research side (specs/REQUESTS.md)', '']
tally = Tally('Requests')
for l in read('specs/REQUESTS.md').splitlines():
    if l.startswith('| R-'):
        c = [x.strip() for x in l.strip().strip('|').split(' | ')]
        answered = c[-1].lower().startswith('answered')
        tally.add(f'{c[0]}: {c[1]}', 'answered' if answered else '', note='' if answered or c[-1].startswith('Open') else 'no status recorded')
counts.append(tally)
body.append('')

# 6. What the code itself says is not built: ticked only by hand, when built.
body += ['## 6. Things the code says are not built', '']
tally = Tally('Stated gaps in the code')
for f in sorted(sources):
    for l in sources[f].splitlines():
        if re.search(r'//.*\b(not built|Not built|not implemented|Not implemented|not reproduced|Not reproduced|TODO|FIXME)\b', l):
            tally.add(f'{os.path.basename(f)}: ' + l.strip().lstrip('/!').strip())
counts.append(tally)

total = sum(c.total for c in counts)
auto = sum(c.auto for c in counts)
by_hand = sum(c.hand for c in counts)
head = ['# Spec checklist', '', 'Everything the specs describe, written by `scripts/spec_list.py`. **An automatic tick means a rule found evidence, not that anyone played it.** The tag on each ticked line names the rule. An unticked line may still be built: the rules are strict and only see plain text, so most built behaviour has no automatic tick. To confirm something yourself, change `[ ]` to `[x]`; a rerun keeps it while the line\'s text is unchanged.', '', f'**{total} items: {auto} ticked by a rule, {by_hand} ticked by hand, {total - auto - by_hand} open.**', '', '| Rule | Ticks | What it found |', '| --- | --- | --- |']
why = {'cited in code': 'The game code names this spec', 'title named in code': 'A code comment naming this spec also names the section\'s title', 'address in code': 'Every boot-data address on the line is read by the code', 'module does nothing': 'The spec says the module does nothing, and it has no code', 'code found': 'The level module has code', 'numbers match the pack code': 'At least two flag, line, movie, item, locator or camera numbers on the line, all found in that pack\'s code', 'a status note traces it': 'A status note says the step was traced', 'story check passed by teleport': 'The goal\'s Done condition held in the story check', 'answered': 'A spec answered the request'}
for r, n in tally_rules.most_common():
    head.append(f'| {r} | {n} | {why[r]} |')
head += ['', '| Part | Items | By a rule | By hand |', '| --- | --- | --- | --- |']
for c in counts:
    head.append(f'| {c.name} | {c.total} | {c.auto} | {c.hand} |')
open(OUT, 'w', encoding='utf-8', newline='\n').write('\n'.join(head + [''] + body) + '\n')
print(total, 'items,', auto, 'by a rule,', by_hand, 'by hand')
print(tally_rules.most_common())
