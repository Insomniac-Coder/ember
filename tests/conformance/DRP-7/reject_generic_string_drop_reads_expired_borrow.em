#$ test: compile-fail
#$ rules: DRP-7
# The Plain control differs only by a destructor that reads the borrowed view.

struct Holder[T]:
    borrowed: str
    marker: T

    fn drop(mut self):
        println(self.borrowed)

fn main():
    _outer: Option[Holder[i32]] = None
    if true:
        value: String = "drop"
        _outer = Some(Holder(value.as_str(), 0)) #$ error[E3060]:
    println(1)
