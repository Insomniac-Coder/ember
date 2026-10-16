#$ test: compile-fail
#$ rules: STR-6
#$ error[E2100]: field `label` is not definitely initialized by struct `init`
#$ error[E2100]: `self` is used before all struct fields are initialized
# `[STR-6]` — every field without a default is assigned on every path of `init`, and before `self`
# is used as a whole (`E2100`).

struct P:
    x: int
    label: String

    fn init(mut self, x: int):
        self.x = x
        if x > 0:
            self.label = "positive"

    fn describe(self) -> String:
        return f"{self.x}"

struct Q:
    x: int
    label: String

    fn init(mut self, x: int):
        self.x = x
        println(self.describe())
        self.label = "q"

    fn describe(self) -> String:
        return f"{self.x}"

fn main():
    println(P(1).x, Q(2).x)
