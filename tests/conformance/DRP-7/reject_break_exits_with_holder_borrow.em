#$ test: compile-fail
#$ rules: DRP-7
# The break ends the loop-local String while the outer Holder remains.

struct Holder[T]:
    borrowed: str
    marker: T

    fn drop(mut self):
        println(self.borrowed)

fn main():
    _outer: Option[Holder[i32]] = None
    while true:
        source: String = "break"
        _outer = Some(Holder(source.as_str(), 0)) #$ error[E3060]:
        break
