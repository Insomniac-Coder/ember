#$ test: compile-fail
#$ rules: R8, R13
#$ help: println(v)
## R8 (b), (g): a name opened with `exists as` is a read-only view of what the `Option` holds; replacing
## the `Option` frees that value, which ends the name (E3060 at a later use).

fn main():
    slot: Option[String] = Some(String("key"))
    if slot exists as v:
        slot = None
        println(v)    #$ error[E3060]: `slot = None` (line 10) ends the text `v` reads (`if slot exists as v:`, line 9), and `v` is used on line 11; print it first: write `println(v)` above line 10
