#$ test: compile-fail
#$ rules: UNS-10b, FN-1
#$ not-help: UnsafeCell

fn write_borrowed(value: i32):
    value = 2 #$ error[E3023]: cannot mutate borrowed parameter `value`

fn main():
    write_borrowed(1)
