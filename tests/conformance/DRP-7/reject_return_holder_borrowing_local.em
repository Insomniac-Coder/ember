#$ test: compile-fail
#$ rules: DRP-7
# The returned Holder would run its destructor after this local String ends.

[T]
struct Holder:
    borrowed: StrView
    marker: T

    fn drop():
        println(this.borrowed)

fn make() -> Holder[i32]:
    source: String = "local"
    return Holder(source.as_str(), 0) #$ error[E3060]:

fn main():
    _holder = make()
