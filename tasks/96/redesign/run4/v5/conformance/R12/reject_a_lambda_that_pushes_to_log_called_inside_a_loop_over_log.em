#$ test: compile-fail
#$ rules: R12, R5, R8
#$ help: move line 16, `click()`, below the loop
## Verdict 86, R12 (rules.md lines 662-672): "A lambda writes nothing about what it changes"; "wherever a
## function value is called, the compiler counts the changes of every function the program can put in
## that place". The lambda pushes to `LOG`, so `click()` inside a loop over `LOG` is refused (R8 (c),
## E3020), the message naming the chain (R18). rules.md section 3's line 1065 message shape.

static mut LOG: Array[String] = []

fn main():
    LOG.push(String("start"))
    click = fn(): LOG.push(String("clicked"))
    for line in LOG:
        println(line)
        click()    #$ error[E3020]: `click()` (line 16) pushes to `LOG` (it runs the function made on line 13) while this loop walks `LOG` (line 14); call it after the loop: move line 16, `click()`, below the loop
    println(LOG.len())
