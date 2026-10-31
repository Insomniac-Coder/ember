#$ test: compile-fail
#$ rules: DRP-7
# The break ends the loop-local String while the outer Holder remains.

[T]
struct Holder:
    borrowed: StrView
    marker: T

    fn drop():
        println(this.borrowed)

fn main():
    _outer: Option[Holder[i32]] = None
    while true:
        source: String = "break"
        _outer = Some(Holder(source.as_str(), 0)) #$ error[E3060]:
        break
