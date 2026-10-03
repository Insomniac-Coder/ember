#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: TXT-2, TXT-10
# The unchecked decoder is only available while checking std's implementation.
# User text cannot bypass UTF-8 boundary and exhaustion invariants with a cursor.
fn main():
    at = 1
    "é".next_char(at) #$ error[E1010]: `str` has no method named `next_char`
