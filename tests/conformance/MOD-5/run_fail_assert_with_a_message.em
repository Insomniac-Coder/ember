#$ test: run-fail
#$ rules: MOD-5
#$ profiles: debug, release, shipping
#$ panics: the list is empty

fn main():
    xs: Array[int] = []
    assert(xs.len() > 0, "the list is empty")
