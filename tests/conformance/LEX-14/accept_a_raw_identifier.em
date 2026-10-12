#$ test: run-pass
#$ rules: LEX-14
#$ stdout: 3
## `[LEX-14]` — `r#name` uses a keyword as a name, as an imported C field
## called `type` needs.

struct Token:
    r#type: i32

fn main():
    t = Token(3)
    println(t.r#type)
