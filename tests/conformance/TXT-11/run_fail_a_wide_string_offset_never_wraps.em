#$ test: run-fail
#$ rules: TXT-11, TXT-4, TYP-31, HEAP-8
#$ profiles: debug, release, shipping
#$ panics: String size or byte offset is negative or too large

fn main():
    s = String.from("abc")
    at: u128 = 18446744073709551617
    s.insert(at, 'x')
