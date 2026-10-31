#$ test: run-pass
#$ rules: TYP-17, TYP-18, MONO-1

interface Scored:
    fn score() -> i32

struct Good:
    value: i32

extend Good implements Scored:
    fn score() -> i32:
        return this.value

struct Reader:
    marker: i32

    [T: needs Scored]
    fn read(value: T) -> i32:
        return value.score()

fn main():
    reader = Reader(0)
    println(reader.read(Good(47)))
#$ stdout: 47
