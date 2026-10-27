#$ test: run-pass
#$ rules: R12, R5, R8
#$ stdout: start
#$ stdout: 2
## Verdict 86, R12 (rules.md lines 662-672): a lambda that pushes to `LOG` writes nothing about it, and
## neither does `main`, which runs it; the compiler works out the change from the body. The walk of `LOG`
## ends first, then `click()` runs. The nearest allowed program to
## reject_a_lambda_that_pushes_to_log_called_inside_a_loop_over_log.

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    click = fn(): LOG.push(String("clicked"))
    for line in LOG:
        println(line)
    click()
    println(LOG.len())
