#$ test: run-pass
#$ rules: TYP-25, EXP-1
#$ profiles: debug, release, shipping
#$ stdout: 12
#$ 1
#$ 32

fn encode(left: i32, right: i32) -> i32:
    return left * 10 + right

fn choose[T](left: T, right: T) -> T:
    return left

fn bump(mut value: i32) -> i32:
    value = value + 1
    return value

fn main():
    println(encode(right=2, left=1))
    println(choose[i32](right=2, left=1))
    state = 1
    println(encode(right=bump(state), left=bump(state)))
