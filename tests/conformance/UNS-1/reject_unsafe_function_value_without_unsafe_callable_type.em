#$ test: compile-fail
#$ rules: UNS-1, FN-6
#$ profiles: debug
#$ error[E0900]: an `unsafe fn` cannot become a safe callable value yet

unsafe fn increase(x: i32) -> i32:
    return x + 1

fn main():
    callback: fn(i32) -> i32 = increase
