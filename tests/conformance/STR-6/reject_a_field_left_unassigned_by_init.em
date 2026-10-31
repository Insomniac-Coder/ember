#$ test: compile-fail
#$ rules: STR-6
#$ error[E2100]: field `label` is not definitely initialized by struct `init`
#$ error[E2100]: `this` is used before all struct fields are initialized
# `[STR-6]` — every field without a default is assigned on every path of `init`, and before `self`
# is used as a whole (`E2100`).

struct P:
    x: int
    label: String

    fn init(x: int):
        this.x = x
        if x > 0:
            this.label = "positive"

    fn describe() -> String:
        return f"{this.x}"

struct Q:
    x: int
    label: String

    fn init(x: int):
        this.x = x
        println(this.describe())
        this.label = "q"

    fn describe() -> String:
        return f"{this.x}"

fn main():
    println(P(1).x, Q(2).x)
