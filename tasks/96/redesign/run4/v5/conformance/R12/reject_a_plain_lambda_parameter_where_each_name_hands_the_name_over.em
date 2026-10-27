#$ test: compile-fail
#$ rules: CLO-3
#$ help: fn(owns s) => println(s.len())
## design rules: R12, R4
## Verdict 71, R12 (rules.md lines 676-680): "A lambda's parameter says what it receives, matching the
## function type it is given to": `each_name` hands each name over (`fn(owns String)`), so a lambda that
## writes `fn(s)` there is refused (E2020). rules.md section 3's line 1092 second message, word for word.
## The nearest allowed program is accept_each_name_hands_each_name_to_a_lambda_that_owns_it.

fn each_name(owns f: fn(owns String)):
    f(String("ann"))

fn main():
    names: Array[String] = [String("a")]
    for n in names:
        println(n)
    each_name(fn(owns s): names.push(s))
    each_name(fn(s) => println(s.len()))    #$ error[E2020]: `each_name` hands each name over (`fn(owns String)`), so this function receives it as its own: write `fn(owns s) => println(s.len())`
    println(names.len())
