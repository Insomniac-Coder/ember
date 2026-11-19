#$ test: compile-fail
#$ rules: OWN-3

fn pick(give: bool):
    s = String("a")
    if give:
        _t = s
    println(s)    #$ error[E3040]

fn main():
    pick(true)
