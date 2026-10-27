#$ test: run-pass
#$ rules: R3
#$ stdout: 1
## R3: `match owns e:` hands the value over to the `match`, so a branch's pattern name owns the
## payload and may hand it on.

enum Msg:
    Text(body: String)
    Quit

fn main():
    m = Msg.Text(String("hi"))
    bodies: Array[String] = []
    match owns m:
        Msg.Text(body):
            bodies.push(body)
        Msg.Quit:
            pass
    println(bodies.len())
