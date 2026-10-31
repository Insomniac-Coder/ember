#$ test: run-pass
#$ rules: DRP-7
# Declare the String first so the Holder's destructor runs before its source.

[T]
struct Holder:
    borrowed: StrView
    marker: T

    fn drop():
        println(this.borrowed)

fn main():
    source: String = "still live"
    _holder: Option[Holder[i32]] = Some(Holder(source.as_str(), 0))
    println(1)
#$ stdout: 1
#$ still live
