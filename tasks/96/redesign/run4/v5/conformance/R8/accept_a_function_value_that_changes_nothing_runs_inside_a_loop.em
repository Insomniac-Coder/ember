#$ test: run-pass
#$ rules: R8, R12
#$ stdout:
#$ ping
#$ start
## R8 (c), R12: what a call through `hook` changes is worked out from every function the program can
## put there (verdict 86): only `ping`, which changes nothing, so calling it inside a loop over `LOG` is
## allowed. The nearest program to `reject_a_call_through_a_function_value_that_changes_the_walked_list.em`.

static mut LOG: Array[String] = []

fn replay(hook: fn()):
    for line in LOG:
        hook()
        println(line)

fn ping():
    println("ping")

fn main():
    LOG.push(String("start"))
    replay(ping)
