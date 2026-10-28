#$ test: run-pass
#$ rules: FN-4, LT-1
#$ stdout: job
#$ stdout: 7
## design rules: R5, R6
## R5: the nearest allowed program: each first line in R5's order, parameters, `owns this`, `-> T`,
## `from` (`fn label(prefix: StrView) -> StrView from prefix:`, `fn finish() owns this -> u64:`).

class Job:
    id: u64

    fn label(prefix: StrView) -> StrView from prefix:
        return prefix.trim()

    fn finish() owns this -> u64:
        return this.id

fn main():
    j = Job(id=7)
    println(j.label(" job "))
    println(j.finish())
