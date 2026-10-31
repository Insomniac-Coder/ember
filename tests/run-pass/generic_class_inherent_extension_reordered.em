#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42

[A, B]
class Pair:
    first: A
    second: B

extend[Left, Right] Pair[Right, Left]:
    fn first_value() -> Right:
        return this.first

fn main():
    pair = Pair[i32, bool](42, false)
    println(pair.first_value())
