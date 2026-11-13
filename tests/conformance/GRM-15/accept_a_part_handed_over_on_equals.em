#$ test: run-pass
#$ rules: GRM-15, OWN-11, BRW-12
#$ stdout: x
#$ stdout: 3
## `[GRM-15]` — before a part on the right of `=` (`body = owns req.body`) the word hands the part over
## into the new name (`[OWN-11]`), where `=` alone makes a view (`[BRW-12]`); the rest stays usable.

struct Req:
    body: String = String("x")
    n: int = 3

fn main():
    req = Req()
    body = owns req.body
    println(body)
    println(req.n)
