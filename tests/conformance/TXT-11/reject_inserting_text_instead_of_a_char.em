#$ test: compile-fail
#$ rules: TXT-11, TYP-5
#$ profiles: debug, release, shipping

fn main():
    s = String()
    s.insert(0, "é")  #$ error[E2020]: expected `char`, found `str`
