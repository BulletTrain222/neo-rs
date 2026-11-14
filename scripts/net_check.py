"""Online play check (docs/MULTIPLAYER.md): a host and a joiner window over 127.0.0.1.

Starts the host (on the autopilot unless --still), waits until it hosts, starts the joiner
(from the title screen), lets them play, closes both, and reads their logs: the join, the
welcomes, the events made and acknowledged, the effect objects and music on each side, the
state sizes, and any panic. Exits 1 on a failed check.

    python scripts/net_check.py [--seconds 60] [--still] [--drop 20] [--delay 100]
        [--host-env K=V ...] [--join-env K=V ...] [--host-args "--pack 909 ..."]

Logs go to private/nettest/host.log and join.log. Opens game windows (they need not be seen).
"""
import argparse
import os
import re
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXE = os.path.join(ROOT, 'target', 'release', 'neopets-app.exe')
OUT = os.path.join(ROOT, 'private', 'nettest')
ANSI = re.compile(r'\x1b\[[0-9;]*m')


def kill_all():
    subprocess.run(['taskkill', '/IM', 'neopets-app.exe', '/F'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def read(path):
    try:
        with open(path, encoding='utf-8', errors='replace') as f:
            return ANSI.sub('', f.read())
    except FileNotFoundError:
        return ''


def wait_for(path, text, limit):
    end = time.time() + limit
    while time.time() < end:
        if text in read(path):
            return True
        time.sleep(1)
    return False


def env_with(pairs, base):
    env = dict(os.environ)
    env.update(base)
    for p in pairs or []:
        k, _, v = p.partition('=')
        env[k] = v
    return env


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--seconds', type=int, default=60)
    ap.add_argument('--still', action='store_true', help='the host does not use the autopilot')
    ap.add_argument('--drop', type=int, default=0, help='percent of received messages dropped (both sides)')
    ap.add_argument('--delay', type=int, default=0, help='milliseconds each received message is held (both sides)')
    ap.add_argument('--host-env', nargs='*')
    ap.add_argument('--join-env', nargs='*')
    ap.add_argument('--host-args', default='', help='more arguments for the host, for example "--pack 909 --flag 110"')
    a = ap.parse_args()

    os.makedirs(OUT, exist_ok=True)
    host_log, join_log = os.path.join(OUT, 'host.log'), os.path.join(OUT, 'join.log')
    common = {'NEOPETS_TRACE_NETEV': '1', 'NEOPETS_NO_MOVIES': '1'}
    if a.drop:
        common['NEOPETS_NET_DROP'] = str(a.drop)
    if a.delay:
        common['NEOPETS_NET_DELAY'] = str(a.delay)
    kill_all()
    time.sleep(1)
    host_args = [EXE, '--play'] + ([] if a.still else ['--autoplay']) + a.host_args.split()
    with open(host_log, 'w') as hf:
        subprocess.Popen(host_args, cwd=ROOT, stdout=hf, stderr=subprocess.STDOUT, env=env_with(a.host_env, dict(common, NEOPETS_NET_HOST='1')))
    failures = []
    if not wait_for(host_log, 'net: hosting', 240):
        failures.append('the host never started hosting')
    else:
        with open(join_log, 'w') as jf:
            subprocess.Popen([EXE, '--play', '--title'], cwd=ROOT, stdout=jf, stderr=subprocess.STDOUT, env=env_with(a.join_env, dict(common, NEOPETS_NET_JOIN='127.0.0.1')))
        if not wait_for(join_log, 'net: welcome 1', 60):
            failures.append('the joiner was never welcomed')
        else:
            time.sleep(a.seconds)
    kill_all()
    time.sleep(1)

    h, j = read(host_log), read(join_log)
    report = []
    if 'panicked' in h or 'panicked' in j:
        failures.append('a window panicked')
    report.append('joined: ' + ('yes' if 'joined' in h else 'no'))
    welcomes = re.findall(r'net: welcome (\d+)', j)
    report.append(f'welcomes on the joiner: {welcomes}')
    ev = re.findall(r'NETEV host tick (\d+): (\d+) events made, (\d+) acknowledged, (\d+) waiting; state (\d+) bytes', h)
    if ev:
        t, made, acked, waiting, size = map(int, ev[-1])
        sizes = [int(e[4]) for e in ev]
        report.append(f'events: {made} made, {acked} acknowledged, {waiting} waiting (host tick {t})')
        report.append(f'state size: largest {max(sizes)} bytes, last {size}')
        if made > 0 and waiting > 200:
            failures.append(f'{waiting} events still waiting')
    elif welcomes:
        failures.append('no event lines from the host')
    bw = re.findall(r'NETEV host tick (\d+): .*; sent (\d+) received (\d+) bytes', h)
    if len(bw) >= 2:
        (t0, o0, i0), (t1, o1, i1) = [tuple(map(int, x)) for x in (bw[-2], bw[-1])]
        secs = max(t1 - t0, 1) / 60
        report.append(f'bandwidth (host, last {secs:.0f} s, with UDP and IP headers): out {(o1 - o0) / secs / 1024:.1f} KB/s, in {(i1 - i0) / secs / 1024:.1f} KB/s')
    played = re.findall(r'NETEV joiner tick \d+: (\d+) host events played', j)
    if played:
        report.append(f'joiner played {played[-1]} events')
    hf = re.findall(r'NETFX host tick (\d+): .*?(\d+) objects.*?music (\[.*?\]) bank', h)
    jf = re.findall(r'NETFX joiner tick (\d+)[^:]*: .*?(\d+) objects.*?music (\[.*?\]) bank', j)
    if welcomes and a.seconds >= 20 and not (hf and jf):
        failures.append('no effect and music lines to compare')
    if hf and jf:
        report.append(f'effect objects: host {hf[-1][1]}, joiner {jf[-1][1]}')
        report.append(f'music: host {hf[-1][2]}, joiner {jf[-1][2]}')
        if hf[-1][2] != jf[-1][2]:
            # The last lines may straddle a change; compare the set of music seen.
            if not set(x[2] for x in hf) & set(x[2] for x in jf):
                failures.append('the music never matched')
    # The world sync: every revision the joiner received must equal the host's of that number.
    hr = {int(m.group(1)): (m.group(2), m.group(3)) for m in re.finditer(r'NETREV host (\d+) items Some\((\d+)\) broken Some\((\d+)\)', h)}
    jr = {int(m.group(1)): (m.group(2), m.group(3)) for m in re.finditer(r'NETREV joiner (\d+) items (\d+) broken (\d+)', j)}
    if hr and jr:
        common = set(hr) & set(jr)
        bad = [k for k in common if hr[k] != jr[k]]
        report.append(f'world revisions: host {len(hr)}, joiner received {len(jr)}, {len(common) - len(bad)} of {len(common)} equal')
        if bad:
            failures.append(f'world revisions differ: {sorted(bad)[:5]}')
        if max(hr) != max(jr) and a.drop == 0:
            failures.append(f'the last world revision ({max(hr)}) never reached the joiner (it has {max(jr)})')
    # The animated objects: the joiner's last list must be one the host had.
    ho = [m.group(1) for m in re.finditer(r'NETFX host tick \d+:[^\n]*objects (\[.*?\]), monster', h)]
    jo = [m.group(1) for m in re.finditer(r'NETFX joiner tick \d+[^:]*:[^\n]*objects (\[.*?\]), monster', j)]
    if ho and jo:
        if jo[-1] not in set(ho):
            failures.append('the joiner ended with an object list the host never had')
        else:
            report.append('animated objects: the joiner\'s match the host\'s')
    # Characters the host has and the joiner lacks.
    miss = re.findall(r'NETFX joiner tick \d+ \(missing Some\((\d+)\)\)', j)
    if miss:
        report.append(f'characters missing on the joiner at the end: {miss[-1]}')
    for line in re.findall(r'net: [^\n]*', h + '\n' + j):
        if 'timed out' in line or 'left' in line:
            report.append('note: ' + line)
    for r in report:
        print(r)
    for f in failures:
        print('FAIL ' + f)
    print('PASS' if not failures else 'FAILED')
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
