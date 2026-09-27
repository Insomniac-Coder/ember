#$ test: run-pass
#$ rules: FN-5, MOD-3, TYP-18
#$ stdout: 13 9
# An imported default resolves names in its declaring module, even when the
# caller has a same-named constant. The generic default uses the concrete T.

from support.defaults import choose, mirror

const SHIFT: int = 100

fn main():
    println(choose(3), mirror(9))
