#$ test: run-pass
#$ rules: BRW-16, GRM-44
#$ stdout: key
## design rules: R8, R13
## R8 (b), (g): the way out E3060's message gives: the opened name is used before the line that
## replaces the `Option`.

fn main():
    slot: Option[String] = Some(String("key"))
    if slot exists as v:
        println(v)
        slot = None
