#$ test: compile-fail
#$ rules: GRM-31, TYP-32
#$ error[E2260]: a `some` type is not written in an interface's method: each implementation would choose its own
#$ error[E2260]: a `some` type is not written in an interface's method: each implementation would choose its own
# `[GRM-31]` (ODR-144) — a method an interface declares, with or without a default body, has no one
# body to choose a `some` type: each implementation would choose its own. An associated type bounded
# by the interface says what each returns.

interface Listing:
    fn items(self) -> some Iterator[Item = int]

    fn firsts(self) -> some Iterator[Item = int]:
        return self.items()

fn main():
    pass
