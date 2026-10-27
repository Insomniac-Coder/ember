#$ test: run-pass
#$ rules: OWN-8
#$ stdout: 1
## design rules: R3
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
