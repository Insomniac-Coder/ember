#$ test: compile-fail
#$ rules: DRP-7
# Continue ends this iteration's String while the outer Holder remains; the
# next iteration would also replace the still-borrowed source local.

struct Holder[T]:
    borrowed: str
    marker: T

    fn drop(mut self):
        println(self.borrowed)

fn main():
    _outer: Option[Holder[i32]] = None
    count: i32 = 0
    while count < 1:
        source: String = "continue" #$ error[E3021]: `source` cannot be written while it is borrowed
        _outer = Some(Holder(source.as_str(), 0)) #$ error[E3060]:
        count += 1
        continue
