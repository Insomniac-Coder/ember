#$ test: compile-fail
#$ rules: TXT-10, BRW-1
#$ profiles: debug, development, release

fn main():
    text = String.from("left:right")
    match text.split_once(":"):
        Some((before, after)):
            text.clear()
            println(before, after)    #$ error[E3060]
        None:
            println("none")
