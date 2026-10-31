#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9
#$ profiles: debug
#$ error[E7001]: field `Holder_Local.value` has type `Local`, which is not Send or Sync

class Local:
    value: int = 1

@sync
[T]
class Holder:
    value: T

    fn init(value: T):
        this.value = value

fn main():
    holder = Holder(Local())
    println(holder.value.value)
