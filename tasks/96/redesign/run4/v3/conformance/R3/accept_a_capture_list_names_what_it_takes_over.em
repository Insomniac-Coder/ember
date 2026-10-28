#$ test: run-pass
#$ rules: R3, R12
#$ stdout: done
#$ 0
## R3, R12: the nearest program that is allowed: `task` captures nothing, and `job` says on the
## captured name that it takes `report` over (AN-47's NEW).

@derive(Clone)
class Report:
    lines: Array[String] = []

fn submit(owns r: Report):
    println(r.lines.len())

fn main():
    report = Report()
    task = fn() => println("done")
    task()
    job = fn() with owns report: submit(report)
    job()
