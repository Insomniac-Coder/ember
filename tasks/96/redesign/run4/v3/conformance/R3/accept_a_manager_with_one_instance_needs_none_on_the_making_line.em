#$ test: run-pass
#$ rules: R3
#$ stdout: 1
## R3: a manager with one instance (`Global`, `System`) needs none on the making line, and costs no
## word in the value.

from std.mem import System

struct Logbook:
    names: Array[String] using System

fn main():
    b = Logbook(names=[String("a")])
    println(b.names.len())
