#$ test: run-pass
#$ rules: MOD-2, STR-1
# The readable half of the matrix from another module: `pub` reads, and so
# does `pub(package)` — a build is one package until `[MAN-2]`'s dependency
# graph exists, so there is nothing for it to be invisible to yet. `pub(read)`
# reads here too; what it refuses is the write, which is a separate test.

from support.access import make, secret_of, retitle

fn main():
    p = make(1, 2, 3, 4)
    println(p.w)
    println(p.h)
    println(p.title)
    println(secret_of(p))
    q = retitle(p, 9)
    println(q.title)
#$ stdout: 1
#$ 2
#$ 3
#$ 4
#$ 9
