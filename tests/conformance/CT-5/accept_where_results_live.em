#$ test: run-pass
#$ rules: CT-5, STA-3, CT-1
#$ stdout: [2, 3, 5, 7, 0]
#$ [2, 3, 5, 7, 0]
#$ ['n0', 'n1', 'n2'] main [0.5, 1.5] 19
#$ 2
# `[CT-5]` — a value that owns heap memory: `comptime(e)` clones it into a fresh allocation at each
# evaluation (so pushing to one leaves the next as it was); a `static` no safe code can reach
# through `ref mut` is constant data in the image, read with no initialisation (so a `@noalloc
# @nosync` function may read it); a `static` holding a `Mutex` is initialised on first access.

from std.sync import Mutex

fn primes(n: int) -> Array[int]:
    out: Array[int] = []
    k = 2
    while out.len() < n:
        prime = true
        for p in out:
            if k % p == 0:
                prime = false
        if prime:
            out.push(k)
        k += 1
    return out

fn names() -> Array[String]:
    out: Array[String] = []
    for i in range(3):
        out.push(f"n{i}")
    return out

struct Config:
    title: String
    weights: Array[f64]
    level: int

fn config() -> Config:
    return Config("main", [0.5, 1.5], 3)

static NAMES: Array[String] = names()
static CONFIG: Config = config()
static COUNTS: Mutex[Array[int]] = Mutex([1])

@noalloc
@nosync
fn total() -> int:
    t = 0
    for n in NAMES:
        t += n.len()
    for w in CONFIG.weights:
        t += w as int
    return t + CONFIG.level * 4

fn main():
    for _ in range(2):
        p = comptime(primes(4))
        p.push(0)
        println(p)
    println(NAMES, CONFIG.title, CONFIG.weights, total())
    with g = COUNTS.lock():
        g.push(2)
        println(g.len())
