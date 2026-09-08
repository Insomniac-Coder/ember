#$ test: run-fail
#$ rules: IV.3
#$ panics: index 4 is out of bounds for a length of 4

fn main():
    xs = [1, 2, 3, 4]
    i: usize = 0
    while i < 5:
        println(xs[i])
        i = i + 1
