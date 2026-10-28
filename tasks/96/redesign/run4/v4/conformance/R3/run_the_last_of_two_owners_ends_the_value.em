#$ test: run-pass
#$ rules: R3, R9
#$ stdout: a ended
#$ end one
#$ b ended
## R3, R9 and `V4/rules.md` section 2 ("a count +1 / -1, a `shares` owner made or ended"): a `shares`
## value lives until the last of its owners ends: ending the first owner only subtracts one, and the
## object ends with the second, once. Verified by its output and a clean AddressSanitizer and leak
## run.

import std.mem

class Res:
    name: String

    fn drop():
        println("end", this.name)

fn main():
    a = Res(name=String("one"))
    b = shares a
    mem.drop(a)
    println("a ended")
    mem.drop(b)
    println("b ended")
