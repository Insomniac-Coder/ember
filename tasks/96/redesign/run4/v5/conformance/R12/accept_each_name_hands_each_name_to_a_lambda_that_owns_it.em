#$ test: run-pass
#$ rules: CLO-3, BRW-18, CTL-2
#$ stdout: a
#$ stdout: 3
#$ stdout: 2
## design rules: R12, R4, R5
## Verdicts 71 and 86, R12 (rules.md lines 676-680, 703-711): a lambda's parameter says what it
## receives, matching the function type: `each_name` hands each name over (`fn(owns String)`), so the
## lambdas write `fn(owns s)`; the first call pushes to `main`'s own `names`, after the loop over it,
## the second changes nothing, and nothing on any first line says so. AN-56's NEW program. The nearest
## allowed program to reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over and to
## reject_each_name_inside_a_loop_over_the_list_its_function_pushes_to.

fn each_name(owns f: fn(owns String)):
    f(String("ann"))

fn main():
    names: Array[String] = [String("a")]
    for n in names:
        println(n)
    each_name(fn(owns s): names.push(s))
    each_name(fn(owns s) => println(s.len()))
    println(names.len())
