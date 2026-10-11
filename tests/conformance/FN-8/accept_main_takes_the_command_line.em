#$ test: run-pass
#$ rules: FN-8
#$ profiles: debug, release, shipping
#$ args: héllo
#$ args: two words
#$ args: --dash
#$ stdout: 4
#$ [héllo]
#$ [two words]
#$ [--dash]
# D-548 — `main` may take `args: Array[String]`, the command line decoded as
# UTF-8; the first is the program's name, as in C. `ember run` passes on what
# follows the file, and everything after `--`.

fn main(args: Array[String]):
    println(args.len())
    for i in 1..args.len():
        println(f"[{args[i]}]")
