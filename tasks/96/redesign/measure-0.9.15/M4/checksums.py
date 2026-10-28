"""checksums.py: the expected checksum of every M4 workload, computed independently of the C and Rust
programs (numpy, with float32 / float64 arithmetic in the programs' order; no FMA, as x86-64 SSE2 code
has none). The programs were compiled but never run; runs.txt takes these numbers.
Run: python checksums.py"""
import numpy as np

M = 2**64


class Rng:  # harness.h's h_rand, exactly
    def __init__(self, seed):
        self.x = seed

    def next(self):
        self.x = (self.x * 6364136223846793005 + 1442695040888963407) % M
        return self.x >> 33


def this_tick():
    r = Rng(17)
    hp = np.array([r.next() % 1000 + 1 for _ in range(16384)], dtype=np.int64)
    ids = np.arange(16384, dtype=np.int64)
    total = deaths = logged = 0
    for _ in range(12000):
        hp -= 1
        dead = hp <= 0
        deaths += int(dead.sum())
        hp[dead] = 1000
        logged += int(ids[dead].sum())
        total += int(hp.sum())
    return (total + deaths * 1000003 + logged + 12000) % M


def this_tally():
    r = Rng(23)
    xs = np.array([r.next() % 1000000 for _ in range(65536)], dtype=np.int64)
    frames = 3000
    total = frames * int(xs.sum())
    mx = int(xs.max())
    seen = frames * 65536
    hist = np.bincount(xs & 255, minlength=256).astype(object) * frames
    s = total + mx + seen + sum(int(hist[k]) * (k + 1) for k in range(256))
    return s % M


def this_mix():
    r = Rng(29)
    inp = np.array([((r.next() % 2001) - 1000) for _ in range(65536)], dtype=np.float64) * 0.001
    out = np.zeros(65536, dtype=np.float64)
    peak = 0.0
    for f in range(3000):
        gain = 0.25 * float(1 + f % 4)
        v = inp * gain
        out = out * 0.5 + v
        peak = max(peak, float(v.max()))
    s = int(np.int64(np.trunc(peak * 1e6))) + int((np.trunc(out * 1e6)).astype(np.int64).sum())
    return s % M


def samevar(one):
    f32 = np.float32
    r = Rng(31)
    left = np.zeros(4096, dtype=f32)
    right = np.zeros(4096, dtype=f32)
    for k in range(4096):
        left[k] = f32(r.next() % 1000) * f32(0.001)
        right[k] = f32(r.next() % 1000) * f32(0.001)
    L = left.reshape(256, 16)
    R = right.reshape(256, 16)
    h, q, t3, e = f32(0.5), f32(0.25), f32(0.75), f32(0.125)
    js = np.arange(256)
    for rnd in range(195313):
        if one:
            L = L * h + L * q
            L = L * t3 + e
        else:
            p = (js * 97 + rnd) & 255
            Rp = R[p]
            L = L * h + Rp * q
            R[p] = Rp * t3 + e
    L = L.reshape(-1)
    R = R.reshape(-1)
    a = np.trunc(L * f32(1000000.0)).astype(np.int64).astype(object)
    b = np.trunc(R * f32(1000000.0)).astype(np.int64).astype(object)
    return int(sum(a) + 3 * sum(b)) % M


def respan():
    f32 = np.float32
    r = Rng(37)
    pos = np.zeros(4096, dtype=f32)
    vel = np.zeros(4096, dtype=f32)
    for i in range(4096):
        pos[i] = f32((r.next() % 2001) - 1000)
        vel[i] = f32((r.next() % 2001) - 1000) * f32(0.01)
    dt = f32(0.01)
    step = vel * dt
    for _ in range(117188):
        pos = pos + step
        pos = np.where(pos > f32(1000.0), pos - f32(2000.0), np.where(pos < f32(-1000.0), pos + f32(2000.0), pos))
    return int(np.trunc(pos * f32(1000.0)).astype(np.int64).astype(object).sum()) % M


def cb_parse():
    r = Rng(41)
    vs = [r.next() % 100000 for _ in range(2000000)]
    text_len = sum(len(str(v)) + 1 for v in vs)
    s = 10 * sum(vs) + 20000000 * 7 + max(vs) * 13 + 2500000 * 17
    return s % M, text_len


def cb_once():
    n = 48828 * 1024
    return (n * (n - 1) // 2 + n) % M


if __name__ == "__main__":
    print("this_tick", this_tick())
    print("this_tally", this_tally())
    print("this_mix", this_mix())
    print("samevar one", samevar(True))
    print("samevar two", samevar(False))
    print("respan", respan())
    print("cb_event", 200000000)
    s, tl = cb_parse()
    print("cb_parse", s, "text bytes", tl)
    print("cb_once", cb_once())
