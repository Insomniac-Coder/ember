#$ test: run-pass
#$ rules: THR-1, THR-8, THR-9
#$ profiles: debug
#$ stdout: 5
#$ assert-c: contains(EMBER_TI_SYNC)

@sync
[T]
class Holder:
    value: T

    fn init(value: T):
        this.value = value

fn main():
    holder = Holder(5)
    println(holder.value)
