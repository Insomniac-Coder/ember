#$ test: run-fail
#$ rules: TXT-11, TXT-4, TYP-31, HEAP-8
#$ profiles: debug, release, shipping
#$ panics: out of bounds

fn main():
    s = String.from("é")
    s.remove(2)
