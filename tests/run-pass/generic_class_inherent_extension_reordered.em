#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42

class Pair[A, B]:
    first: A
    second: B

extend[Left, Right] Pair[Right, Left]:
    fn first_value(self) -> Right:
        return self.first

fn main():
    pair = Pair[i32, bool](42, false)
    println(pair.first_value())
