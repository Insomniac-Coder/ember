#$ test: compile-fail
#$ rules: UNS-10b, FN-1
#$ not-help: UnsafeCell

fn write_borrowed(value: i32):
    value = 2 #$ error[E3023]: `value` is the number the caller passed and stays that number for the whole call; for a number of your own write `new_value = 2` in place of line 6

fn main():
    write_borrowed(1)
