#$ test: compile-fail
#$ rules: TXT-11, BRW-1, SPN-1
#$ profiles: debug, release, shipping

fn main():
    s = String.from("abc")
    view = s.as_str()
    s.reserve(4)  #$ error[E3021]: `s` is borrowed here and mutably borrowed elsewhere
    println(view)
