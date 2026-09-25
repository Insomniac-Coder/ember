#$ test: compile-fail
#$ rules: IFC-4
# ODR-040 — `Output = V` binds an associated type only in a bound; an
# implementation states it in its block (`type Output = V`).

struct V:
    x: int

extend V implements Add[Output = V]:    #$ error[E2020]: `Output = …` binds an associated type only in a bound
    fn add(self, rhs: V) -> V:
        return self

fn main():
    pass
