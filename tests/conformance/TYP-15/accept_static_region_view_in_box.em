#$ test: run-pass
#$ rules: TYP-15, LT-3, HEAP-1, DRP-6
# A Box has no bounding region, but a string literal has the static region and
# therefore outlives it. TYP-15 is checked at the stored value, not by banning
# the otherwise-valid Box[str] type.

fn main():
    greeting: Box[str] = Box("hello")
    println(greeting.get())
#$ stdout: hello
