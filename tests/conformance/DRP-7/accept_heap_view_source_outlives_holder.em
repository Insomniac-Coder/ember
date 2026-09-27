#$ test: run-pass
#$ rules: DRP-7
# Declare the String first so the Holder's destructor runs before its source.

struct Holder[T]:
    borrowed: str
    marker: T

    fn drop(mut self):
        println(self.borrowed)

fn main():
    source: String = "still live"
    _holder: Option[Holder[i32]] = Some(Holder(source.as_str(), 0))
    println(1)
#$ stdout: 1
#$ still live
