#$ test: compile-fail
#$ rules: ERR-4, OWN-3
# `map` takes its receiver `owned`: an `Option[String]` is gone after it.

fn main():
    names: Option[String] = Some("a")
    upper = names.map(owns fn(s) => s)
    println(names)    #$ error[E3040]: `names` was handed over on line 7 and is used on line 8; to keep both, hand over a copy of your own: `copy of names`
    println(upper)
