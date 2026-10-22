#$ test: compile-fail
#$ profiles: debug, development, release
#$ rules: TXT-2, TXT-10
# The unchecked decoder is only available while checking std's implementation.
# User text cannot bypass UTF-8 boundary and exhaustion invariants with a cursor.
fn main():
    at = 1
    "é".next_char(at) #$ error[E2072]: `str` has no method named `next_char`
