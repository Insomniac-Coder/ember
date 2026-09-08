#$ test: run-pass
#$ rules: XVIII.4.6

fn classify(n: i32) -> i32:
    result: i32
    if n > 0:
        result = 1
    else:
        result = 0
    return result

fn main():
    # Declared without a value, then assigned before every read.
    total: i32
    total = 0
    total += classify(5)
    total += classify(-5)
    println(total)
#$ stdout: 1
