#$ test: compile-fail
#$ rules: ERR-4, OWN-3
# `map` takes its receiver `owned`: an `Option[String]` is gone after it.

fn main():
    names: Option[String] = Some("a")
    upper = names.map(fn(s) => s)
    println(names)    #$ error[E3050]: `names` is borrowed after it has been moved out of
    println(upper)
