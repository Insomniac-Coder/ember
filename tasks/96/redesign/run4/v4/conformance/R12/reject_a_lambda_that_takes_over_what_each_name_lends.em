#$ test: compile-fail
#$ rules: R12, R4
#$ help: fn(s) changes names: names.push(copy of s)
## Verdict 71, R12 (rules.md lines 627-630): a lambda's parameter matches the function type it is given
## to; `each_name` lends each name read-only (`fn(String)`), so a lambda that takes it over (`fn(owns s)`)
## would hand over a name it was only lent (E2020). rules.md section 3's line 1041 first message, word for
## word (completeness G5).

[C]
fn each_name(owns f: fn(String) changes C) changes C:
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owns s) changes names: names.push(s))    #$ error[E2020]: `each_name` gives each name read-only, so this function cannot take it over; to keep your own, write `fn(s) changes names: names.push(copy of s)`
    println(names.len())
