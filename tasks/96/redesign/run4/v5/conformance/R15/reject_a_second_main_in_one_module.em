#$ test: compile-fail
#$ rules: TYP-26
## design rules: R15
## Section 4: E1030 has one meaning, "two items with the same name in one scope, such as a second
## `main`" (completeness G4; the message is written to R18).

fn main():
    println(1)

fn main():    #$ error[E1030]: `main` is already declared in this module (line 6), and a module has one item of each name; rename this function, for example `fn print_two():`, or remove it
    println(2)
