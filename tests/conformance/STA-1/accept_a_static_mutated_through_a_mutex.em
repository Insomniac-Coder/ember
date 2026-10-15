#$ test: run-pass
#$ rules: STA-1, STA-3, THR-3
#$ stdout: 2 3
# `[STA-1]` — a `static` is immutable; mutation goes through a `Sync` interior type, a `Mutex` here,
# initialised on first access (`[STA-3]`).

from std.sync import Mutex

static REGISTRY: Mutex[Map[String, int]] = Mutex({})

fn record(name: str, n: int):
    with r = REGISTRY.lock():
        r[name.to_string()] = n

fn main():
    record("a", 1)
    record("b", 2)
    with r = REGISTRY.lock():
        println(r.len(), r["a"] + r["b"])
