#$ test: compile-fail
#$ rules: DIA-16
# `[DIA-16]` — a diagnostic that suggests a `shares` or `shares mut` owner has a note naming its cost while the
# program runs: one count per owner made or ended, and one compare with the count's maximum per owner made.

class Counter:
    n: i32

fn main():
    a = shares Counter(0)
    a.n += 1    #$ error[E3023]: a `shares` or `shares mut` owner costs, while the program runs, one count per owner made or ended, and one compare with the count's maximum per owner made
