#$ test: compile-fail
#$ rules: DRP-7
# This differs from the Plain control only by its destructor, which reads the
# borrowed field at scope exit after the inner source has ended.

struct Holder[T]:
    borrowed: ref i32
    marker: T

    fn drop(mut self):
        println(self.borrowed)

fn main():
    _outer: Option[Holder[i32]] = None
    if true:
        value: i32 = 5
        _outer = Some(Holder(ref value, 0)) #$ error[E3060]:
    println(1)
