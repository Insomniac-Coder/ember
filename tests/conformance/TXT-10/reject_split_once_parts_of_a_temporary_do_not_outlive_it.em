#$ test: compile-fail
#$ rules: TXT-10, BRW-8
#$ profiles: debug, release, shipping

fn text() -> String:
    return "left:right"

fn main():
    pair = text().split_once(":")  #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(pair)
