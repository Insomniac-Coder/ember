#$ test: run-pass
#$ rules: TYP-24, TYP-25, EXP-1
#$ profiles: debug, release, shipping
#$ stdout: 12
#$ 1

struct Pair:
    left: i32
    right: i32

extend Pair:
    fn encode(self, left: i32, right: i32) -> i32:
        return left * 10 + right

    fn choose[T](self, left: T, right: T) -> T:
        return left

fn main():
    pair = Pair(0, 0)
    println(pair.encode(right=2, left=1))
    println(pair.choose[i32](right=2, left=1))
