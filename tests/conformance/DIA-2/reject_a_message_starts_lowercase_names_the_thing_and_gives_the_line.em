#$ test: compile-fail
#$ rules: DIA-2, DIA-25
#$ error[E1010]: inside a method its object is `this`
#$ help: write `this.n = 1`
# `[DIA-2]` — a message starts lowercase, has no trailing period, names the thing and says what is
# wrong; `help` says what to do and gives the exact line to write.

struct Counter:
    n: int

    fn bump():
        self.n = 1

fn main():
    c = Counter(n=0)
    c.bump()
