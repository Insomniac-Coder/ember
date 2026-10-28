#$ test: run-pass
#$ rules: R8, R13
#$ stdout: key
## R8 (b), (g): the way out E3060's message gives: the opened name is used before the line that
## replaces the `Option`.

fn main():
    slot: Option[String] = Some(String("key"))
    if slot exists as v:
        println(v)
        slot = None
