#$ test: run-fail
#$ rules: TXT-11, RNG-8, TYP-31
#$ profiles: debug, release, shipping
#$ panics: String size or byte offset is negative or too large

type Count = i128 in -4 ..= 4

fn main():
    text = String.from("abc")
    count: Count = -1
    text.truncate(count)
