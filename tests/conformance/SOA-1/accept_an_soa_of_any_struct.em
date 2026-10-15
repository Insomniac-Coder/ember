#$ test: run-pass
#$ rules: SOA-1, SOA-2
#$ profiles: debug, release
#$ stdout: 3 2.5 hello true
# `[SOA-1]` — any struct gives an `SoA`, with no derive: one column per field, whatever its type
# (an integer, a float, a `String`, a struct, a `bool`).

struct Inner:
    x: f64
    y: f64

struct Mixed:
    count: int
    at: Inner
    label: String
    on: bool

fn main():
    ms = SoA[Mixed].with_capacity(0)
    ms.push(Mixed(3, Inner(2.5, 1.0), "hello".to_string(), true))
    println(ms[0].count, ms[0].at.x, ms[0].label, ms.on[0])
