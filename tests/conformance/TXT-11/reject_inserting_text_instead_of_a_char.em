#$ test: compile-fail
#$ rules: TXT-11, TYP-5
#$ profiles: debug, development, release

fn main():
    s = String()
    s.insert(0, "é")  #$ error[E2020]: expected `char`, found `str`
