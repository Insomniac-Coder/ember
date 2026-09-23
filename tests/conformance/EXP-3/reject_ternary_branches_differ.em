#$ test: compile-fail
#$ rules: EXP-3
#$ profiles: debug
#$ error[E2020]
# Both branches have one type; nothing converts between them implicitly.

fn main():
    q = 5
    x = 1 if q > 3 else "one"
    println(x)
