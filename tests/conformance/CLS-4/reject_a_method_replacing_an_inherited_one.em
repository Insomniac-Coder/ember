#$ test: compile-fail
#$ rules: CLS-4
# `[CLS-4]` (ODR-028) — a method named like an inherited one replaces it:
# over a virtual method it must say `override` (`E2111`), and over a
# non-virtual one it is `E2110`, `override` or not.

open class Base:
    virtual fn speak(self) -> int:
        return 1

    fn plain(self) -> int:
        return 1

class Derived(Base):
    fn speak(self) -> int:              #$ error[E2111]: `speak` replaces an inherited virtual method, so it must say `override`
        return 2

    fn plain(self) -> int:              #$ error[E2110]: `plain` would replace an inherited method that is not virtual
        return 2

fn main():
    pass
