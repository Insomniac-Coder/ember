#$ test: run-pass
#$ rules: THR-1
#$ stdout: 1
#$ stdout: 5
## design rules: R15
## R15: the lines E2221 gives: no view into the frame is held across `yield` (AN-77).

gen fn numbers() -> Generator[int]:
    n = 5
    yield 1
    yield n

fn main():
    for x in numbers():
        println(x)
