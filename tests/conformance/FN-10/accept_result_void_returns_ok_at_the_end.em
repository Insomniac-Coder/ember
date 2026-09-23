#$ test: run-pass
#$ rules: FN-10
#$ profiles: debug, release, shipping
#$ stdout: ok
#$ negative
# A function returning `Result[void, E]` returns `Ok(())` when control reaches
# the end of its body. (Before D-186 it returned an uninitialised value.)

fn check(n: int) -> Result[void, String]:
    if n < 0:
        return Err("negative")

fn main():
    match check(1):
        Ok(_) => println("ok")
        Err(e) => println(e.as_str())
    match check(-1):
        Ok(_) => println("ok")
        Err(e) => println(e.as_str())
