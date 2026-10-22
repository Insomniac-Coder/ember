#$ test: compile-fail
#$ rules: EXP-6, DIA-7, DIA-1
#$ help: `mem.take`
# `[DIA-7]`, `[DIA-1]` (D-822) — a move out of a borrowed place inside a struct literal is reported
# at the value moved (`self.title`), not at the literal's last field, here a default declared
# on another line; the help is shape O2's, with the copy that keeps the caller's value.

struct Report:
    title: String
    count: int
    rows: Array[int] = []

    fn renamed(self) -> Report:
        return Report(title=self.title, count=1)    #$ error[E3013]: cannot move out of a borrowed parameter

fn main():
    r = Report(title="a", count=1)
    println(r.renamed().count)
