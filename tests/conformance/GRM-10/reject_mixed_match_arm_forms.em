#$ test: parse-fail
#$ rules: GRM-10
## `[GRM-10]` — `pattern: block` arms make a statement, `pattern => e` arms an
## expression; one `match` mixing them is `E0103`.

fn main():
    x = 1
    y = match x:
        1 => 10
        _:    #$ error[E0103]: match arms mix statement and expression form
            println(0)
    println(y)
