#$ test: run-pass
#$ rules: THR-1, RC-4, THR-7
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(EMBER_TI_SYNC)
# The type-info flag selects the runtime's atomic count operations.
class Token:
    value: int

    fn init():
        this.value = 42

fn main():
    token = thread_shared Token()
    copy = token
    println(copy.value)
