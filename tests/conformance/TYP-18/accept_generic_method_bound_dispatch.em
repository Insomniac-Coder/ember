#$ test: run-pass
#$ rules: TYP-17, TYP-18, MONO-1

interface Scored:
    fn score(self) -> i32

struct Good:
    value: i32

extend Good implements Scored:
    fn score(self) -> i32:
        return self.value

struct Reader:
    marker: i32

    fn read[T: Scored](self, value: T) -> i32:
        return value.score()

fn main():
    reader = Reader(0)
    println(reader.read(Good(47)))
#$ stdout: 47
