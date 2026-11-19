#$ test: run-pass
#$ rules: TST-3, ATT-6
#$ profiles: debug
#$ stdout: 5
#$ main ran
# `@test` and `@should_panic` are built: a test is an ordinary function the
# program may call too, and `ember run` runs `main`, never a test. `ember test`
# runs each test in a process of its own (the milestones run it on packages).

fn add(a: int, b: int) -> int:
    return a + b

@test
fn adds():
    assert(add(2, 3) == 5)

@test
@should_panic
fn refuses_a_bad_count():
    panic("a bad count")

fn main():
    adds()
    println(add(2, 3))
    println("main ran")
