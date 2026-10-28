#$ test: compile-fail
#$ rules: R12, R4
#$ help: fn(owns s) => println(s.len())
## Verdict 71, R12 (rules.md lines 627-630): "A lambda's parameter says what it receives, matching the
## function type it is given to": `each_name` hands each name over (`fn(owns String)`), so a lambda that
## writes `fn(s)` there is refused (E2020). rules.md section 3's line 1041 second message, word for word.
## The nearest allowed program is accept_each_name_takes_a_change_list.

[C]
fn each_name(owns f: fn(owns String) changes C) changes C:
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owns s) changes names: names.push(s))
    each_name(fn(s) => println(s.len()))    #$ error[E2020]: `each_name` hands each name over (`fn(owns String)`), so this function receives it as its own: write `fn(owns s) => println(s.len())`
    println(names.len())
