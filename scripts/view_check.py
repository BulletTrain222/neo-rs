"""Pictures every saved F8 view (private/views.txt) and compares it with its baseline.

    python scripts/view_check.py --baseline     take the pictures and keep them as the baseline
    python scripts/view_check.py                take them again and report which views changed
    python scripts/view_check.py 12 16          only those views

It opens a game window for each view, with cutscenes off. The result is private/viewcheck/index.html: baseline and new side by
side, biggest change first. Clouds, water and characters move, so a small percentage is normal.
"""
import os, re, shutil, struct, subprocess, sys, zlib

repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
os.chdir(repo)
base = os.path.join('private', 'viewcheck')
exe = os.path.join('target', 'release', 'neopets-app.exe')
args = sys.argv[1:]
keep = '--baseline' in args
only = {a for a in args if a.isdigit()}
os.makedirs(os.path.join(base, 'baseline'), exist_ok=True)
os.makedirs(os.path.join(base, 'new'), exist_ok=True)


def png(path):
    """Width, height, channels and rows of an 8-bit PNG."""
    b = open(path, 'rb').read()
    at, data, w, h, ch = 8, b'', 0, 0, 3
    while at < len(b):
        n, kind = struct.unpack('>I4s', b[at:at + 8])
        body = b[at + 8:at + 8 + n]
        if kind == b'IHDR':
            w, h, _, colour = struct.unpack('>IIBB', body[:10])
            ch = {2: 3, 6: 4, 0: 1}[colour]
        elif kind == b'IDAT':
            data += body
        at += 12 + n
    raw = zlib.decompress(data)
    rows, prev, stride = [], bytearray(w * ch), w * ch
    for y in range(h):
        f = raw[y * (stride + 1)]
        line = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - ch] if i >= ch else 0
            u = prev[i]
            c = prev[i - ch] if i >= ch else 0
            if f == 1:
                line[i] = (line[i] + a) & 255
            elif f == 2:
                line[i] = (line[i] + u) & 255
            elif f == 3:
                line[i] = (line[i] + (a + u) // 2) & 255
            elif f == 4:
                p = a + u - c
                pa, pb, pc = abs(p - a), abs(p - u), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else u if pb <= pc else c)) & 255
        rows.append(line)
        prev = line
    return w, h, ch, rows


def changed(a, b):
    """The percentage of sampled pixels that differ by more than a little."""
    wa, ha, ca, ra = png(a)
    wb, hb, cb, rb = png(b)
    if (wa, ha) != (wb, hb):
        return 100.0
    n = bad = 0
    for y in range(0, ha, 4):
        for x in range(0, wa, 4):
            n += 1
            if any(abs(ra[y][x * ca + k] - rb[y][x * cb + k]) > 24 for k in range(3)):
                bad += 1
    return 100.0 * bad / max(n, 1)


views = []
for line in open(os.path.join('private', 'views.txt')):
    m = re.match(r'view (\d+): pack (\d+) hero (\S+)', line)
    if m and (not only or m[1] in only):
        views.append((m[1], m[2], m[3]))
results = []
for n, pack, hero in views:
    out = os.path.join(base, 'new', f'{n}.png')
    env = {k: v for k, v in os.environ.items() if not k.startswith('NEOPETS_')}
    env.update(NEOPETS_MOVIES='0', NEOPETS_VIEW=n)
    try:
        subprocess.run([exe, '--play', '--pack', pack, '--start', hero, '--screenshot', out, '--screenshot-at', '9'], env=env, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=90)
    except subprocess.TimeoutExpired:
        pass
    old = os.path.join(base, 'baseline', f'{n}.png')
    if keep or not os.path.exists(old):
        shutil.copy(out, old)
        print(f'view {n} (world {pack}): baseline saved')
        results.append((0.0, n, pack))
        continue
    d = changed(old, out)
    results.append((d, n, pack))
    print(f'view {n} (world {pack}): {d:.1f}% changed')
results.sort(reverse=True)
html = ['<!doctype html><meta charset=utf-8><title>View check</title><style>body{font:14px sans-serif;background:#222;color:#eee}img{width:49%}</style><h1>View check</h1>']
for d, n, pack in results:
    html.append(f'<h3>View {n}, world {pack}: {d:.1f}% changed</h3><img src="baseline/{n}.png"> <img src="new/{n}.png">')
open(os.path.join(base, 'index.html'), 'w', encoding='utf-8').write('\n'.join(html))
print('Report:', os.path.join(base, 'index.html'))
