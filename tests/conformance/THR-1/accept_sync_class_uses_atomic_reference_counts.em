#$ test: run-pass
#$ rules: THR-1, RC-4, THR-7
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(EMBER_TI_SYNC)
# The type-info flag selects the runtime's atomic count operations.
@sync
class Token:
    value: int

    fn init(mut self):
        self.value = 42

fn main():
    token = Token()
    copy = token
    println(copy.value)
