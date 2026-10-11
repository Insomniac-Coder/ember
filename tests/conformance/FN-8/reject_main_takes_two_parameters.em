#$ test: compile-fail
#$ rules: FN-8
# ODR-101 — `main` takes at most one parameter, the command line.

fn main(args: Array[String], more: Array[String]): #$ error[E2183]: it takes 2 parameters
    println(args.len() + more.len())
