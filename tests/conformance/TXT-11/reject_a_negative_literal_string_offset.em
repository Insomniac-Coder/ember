#$ test: compile-fail
#$ rules: TXT-11, TYP-31
#$ profiles: debug, release, shipping

fn main():
    s = String.from("abc")
    s.insert(-1, 'x')  #$ error[E2011]: negative literal index
