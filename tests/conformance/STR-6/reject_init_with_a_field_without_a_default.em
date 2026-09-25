#$ test: compile-fail
#$ rules: STR-6
# `[STR-6]` — an `init` that must assign a field with no default needs
# definite-assignment checking (`E2100`), which is not built yet; it is
# refused rather than run half-checked.

struct P:
    x: int

    fn init(mut self, x: int):
        self.x = x

fn main():
    p = P(1)  #$ error[E0900]: `init` on `P`, whose field `x` has no default, is not implemented yet
