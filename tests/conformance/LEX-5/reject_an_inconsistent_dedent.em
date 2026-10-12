#$ test: parse-fail
#$ rules: LEX-5
## `[LEX-5]` — a shallower line pops the indentation stack until equal; one
## that lands between two levels is `E0003`.

fn main():
    if true:
        println(1)
      println(2)    #$ error[E0003]: inconsistent dedent
