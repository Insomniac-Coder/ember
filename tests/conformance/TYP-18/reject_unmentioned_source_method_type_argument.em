#$ test: compile-fail
#$ rules: TYP-18

struct Marker:
    value: i32

    [T]
    fn number() -> i32:
        return this.value

fn main():
    marker = Marker(52)
    println(marker.number()) #$ error[E2060]: cannot tell what `T` is here
