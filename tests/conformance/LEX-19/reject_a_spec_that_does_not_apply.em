#$ test: compile-fail
#$ rules: LEX-19
#$ profiles: debug
#$ error[E2250]: format spec `:d` does not apply to `str`
# A spec that does not apply to the value's type is `E2250`, naming both.

fn main():
    name = "Ann"
    println(f"{name:d}")
