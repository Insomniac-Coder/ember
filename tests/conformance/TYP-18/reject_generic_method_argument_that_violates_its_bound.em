#$ test: compile-fail
#$ rules: TYP-17, TYP-18

interface Scored:
    fn score(self) -> i32

struct Good:
    value: i32

extend Good implements Scored:
    fn score(self) -> i32:
        return self.value

struct Bad:
    value: i32

struct Reader:
    marker: i32

    fn read[T: Scored](self, value: T) -> i32:
        return value.score()

fn main():
    reader = Reader(0)
    println(reader.read(Good(45)))
    println(reader.read(Bad(46))) #$ error[E2040]: `Bad` does not implement `Scored`, which `T` requires
