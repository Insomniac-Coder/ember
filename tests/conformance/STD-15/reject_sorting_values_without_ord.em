#$ test: compile-fail
#$ rules: STD-15, STR-5
#$ profiles: debug
#$ error[E2040]: `Point` does not implement `Ord`, which `sort` needs

@derive(Copy)
struct Point:
    x: int

fn main():
    ps = [Point(x = 2), Point(x = 1)]
    ps.sort()
