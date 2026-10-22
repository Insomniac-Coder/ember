#$ test: run-pass
#$ rules: CLS-1, CLS-3, OWN-2
#$ assert-c: contains(&em_ti_Empty))
#$ stdout: 42

class Empty:
    fn answer(self) -> i32:
        return 42

fn main():
    value = Empty()
    println(value.answer())
