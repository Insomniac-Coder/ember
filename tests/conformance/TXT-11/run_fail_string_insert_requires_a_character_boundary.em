#$ test: run-fail
#$ rules: TXT-11, TXT-4, TYP-31, HEAP-8
#$ profiles: debug, release, shipping
#$ panics: String byte offset is not on a character boundary

fn main():
    s = String.from("é🌶")
    s.insert(1, 'x')
