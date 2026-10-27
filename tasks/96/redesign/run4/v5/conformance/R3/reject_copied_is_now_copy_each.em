#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `names = sorted(m.keys().copy_each())`
## design rules: R3
## R3 and the removed words of `V4/rules.md` section 3: the iterator method `.copied()` is written `.copy_each()`
## (E1010; verdict 33; the `.copied()` row of `V4/vocabulary.md`; completeness gap G8).

fn main():
    m: Map[String, int] = Map()
    names = sorted(m.keys().copied())      #$ error[E1010]: `.copied()` is now `.copy_each()`
    println(names.len())
