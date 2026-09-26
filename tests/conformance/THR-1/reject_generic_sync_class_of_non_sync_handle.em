#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9
#$ profiles: debug
#$ error[E7001]: field `Holder_Local.value` has type `Local`, which is not Send or Sync

class Local:
    value: int = 1

@sync
class Holder[T]:
    value: T

    fn init(mut self, value: T):
        self.value = value

fn main():
    holder = Holder(Local())
    println(holder.value.value)
