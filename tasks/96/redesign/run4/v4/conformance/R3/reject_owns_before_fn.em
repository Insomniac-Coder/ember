#$ test: compile-fail
#$ rules: R3, R12
#$ help: name it on each captured name (`fn() with owns report:`); `task` captures nothing, so write `task = fn() => println("done")`; a name not listed is the real variable
## R3, R12: `owns` before `fn` is refused (E0109): it does not say what the function takes or copies;
## a capture list names it on each captured name (AN-47's message).

@derive(Clone)
class Report:
    lines: Array[String] = []

fn submit(owns r: Report):
    println(r.lines.len())

fn main():
    report = Report()
    task = owns fn() => println("done")    #$ error[E0109]: `owns fn` does not say what the function takes or copies
    task()
    job = fn() with owns report: submit(report)
    job()
