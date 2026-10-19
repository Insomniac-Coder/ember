#$ test: run-fail
#$ rules: ERR-4
#$ profiles: debug, development, release
#$ panics: the config had no port

fn main():
    port: Option[int] = None
    println(port.expect("the config had no port"))
