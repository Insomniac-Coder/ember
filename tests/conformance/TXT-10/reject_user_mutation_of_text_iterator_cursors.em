#$ test: compile-fail
#$ profiles: debug, release, shipping
#$ rules: MOD-2, TXT-10
# Neither iterator lets a caller forge a continuation-byte cursor or pass the end.
fn main():
    chars = "é".chars()
    chars.at = 1 #$ error[E1052]: `at` is private to
    indices = "é".char_indices()
    indices.at = 3 #$ error[E1052]: `at` is private to
