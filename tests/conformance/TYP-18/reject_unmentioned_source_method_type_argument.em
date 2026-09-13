#$ test: compile-fail
#$ rules: TYP-18

struct Marker:
    value: i32

    fn number[T](self) -> i32:
        return self.value

fn main():
    marker = Marker(52)
    println(marker.number()) #$ error[E2060]: cannot tell what `T` is here
