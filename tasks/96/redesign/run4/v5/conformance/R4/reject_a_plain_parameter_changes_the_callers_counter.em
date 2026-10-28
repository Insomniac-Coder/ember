#$ test: compile-fail
#$ rules: BRW-15
#$ help: fn bump(ref mut c: Counter):
## design rules: R4, R5
## R4: what a function may do with a parameter is set by the word in front of its name; a plain
## parameter only reads the caller's value, for every type, a class object included (AN-20).

class Counter:
    n: int = 0

fn bump(c: Counter):
    c.n += 1    #$ error[E3023]: `bump` changes `c.n`, but `c` only reads the caller's counter; to change it write `fn bump(ref mut c: Counter):` and call `bump(ref mut counter)`

fn main():
    counter = Counter()
    bump(counter)
    println(counter.n)
