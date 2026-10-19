#$ test: run-pass
#$ rules: FN-8, FN-1
#$ profiles: debug, development
#$ args: a
#$ stdout: 3
#$ extra
# D-548 — an `owned` `args` is `main`'s to change; a `debug` run's leak check
# finds nothing left over.

fn main(owned args: Array[String]):
    args.push("extra".to_string())
    println(args.len())
    println(args[args.len() - 1])
