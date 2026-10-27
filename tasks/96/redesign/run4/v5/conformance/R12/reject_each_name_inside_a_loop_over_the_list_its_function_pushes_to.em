#$ test: compile-fail
#$ rules: R12, R5, R8
#$ help: move line 17 below the loop
## Verdict 86 (verdict 21 keeps its meaning without the change list), R12 (rules.md lines 703-711): a
## function that runs the function it is given makes that function's changes, worked out from it ("the
## call counts as making what the function given changes"); `each_name` runs a lambda that pushes to
## `names`, so the call inside a loop over `names` is refused (E3020), the message naming the chain
## (R18). The nearest allowed program is accept_each_name_hands_each_name_to_a_lambda_that_owns_it.

fn each_name(owns f: fn(owns String)):
    f(String("ann"))

fn main():
    names: Array[String] = [String("a")]
    for n in names:
        println(n)
        each_name(fn(owns s): names.push(s))    #$ error[E3020]: `each_name(...)` (line 17) adds to `names` (`each_name` runs the function it is given, line 11, which pushes to `names`) while this loop walks `names` (line 15); call it after the loop: move line 17 below the loop
    println(names.len())
