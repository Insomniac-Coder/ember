#$ test: parse-fail
#$ rules: GRM-28, GRM-33
## `[GRM-33]` — a local function is no interface member: it needs a body.

fn main():
    fn nobody(x: int) -> int    #$ error[E0110]: `nobody` is declared without a body
    println(1)
