#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1
## R12 (rules.md lines 622-625), verdict 17: the lambda names `LOG` on its first line, and `main`, which
## runs it, names `LOG` too (R5, rules.md lines 345-348). The nearest allowed program to
## reject_a_lambda_that_pushes_to_log_without_saying_so.

static mut LOG: Array[String] = []

fn main() changes LOG:
    click = fn() changes LOG: LOG.push(String("clicked"))
    click()
    println(LOG.len())
