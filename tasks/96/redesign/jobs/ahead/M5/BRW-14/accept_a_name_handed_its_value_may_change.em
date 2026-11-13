# Clause BRW-14.6: "A name that was handed its value owns it, as a local does, and keeps no place: an
# `owns` parameter (`[FN-1]`), a loop name under `for x in owns e:` (`[CTL-1]`) and a name of the
# branch that runs under `match owns e:` (`[GRM-13]`) may be changed and given a new value with no
# word (`job.retries += 1` under `for job in owns jobs:`; `name = String("unnamed")` on `owns name:
# String` ends the string the caller handed over, as `=` on a local does, `[OWN-3]`), and `kept =
# job` hands it over."
#$ test: run-pass
#$ rules: BRW-14, FN-1, CTL-1, GRM-13, OWN-3
#$ stdout: a 1
#$ stdout: b 1
#$ stdout: xy
#$ stdout: unnamed

struct Job:
    name: String
    retries: int = 0

fn rename(owns name: String) -> String:
    name = String("unnamed")
    return name

fn main():
    jobs = [Job(name=String("a")), Job(name=String("b"))]
    for job in owns jobs:
        job.retries += 1
        kept = job
        println(kept.name, kept.retries)
    o: Option[String] = Some(String("x"))
    match owns o:
        Some(s):
            s += "y"
            println(s)
        None:
            pass
    println(rename(owns String("n")))
