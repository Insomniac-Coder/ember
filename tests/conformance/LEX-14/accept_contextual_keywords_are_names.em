#$ test: run-pass
#$ rules: LEX-14, LEX-15
#$ stdout: 3 4 5
## `[LEX-14]` — only the words of the table are keywords where a name is expected; a contextual
## keyword (`[LEX-15]`) is a name away from its position, and `r#type` uses a keyword as one.

fn main():
    runtime = 3
    needs = runtime + 1
    r#type = needs + 1
    println(runtime, needs, r#type)
