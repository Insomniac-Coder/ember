#$ test: compile-fail
#$ rules: LEX-26
## `[LEX-26]` — a byte string holds bytes, so `\u{…}` in one is `E0009` ("write the UTF-8 bytes,
## `\xF0\x9F\x98\x80`, or a text literal"); a text literal takes `\x` with `00` to `7F` only, since its
## text is UTF-8 (above it is `E0009`: "write the character, `\u{FF}`, or a byte string, `b"\xFF"`"); an
## escape that is none of `escape`'s forms, and a code that is no Unicode scalar value, are `E0009` too.

fn main():
    a = b"\u{1F600}"    #$ error[E0009]: a byte string holds bytes
                        #$ help: write the UTF-8 bytes, `\xF0\x9F\x98\x80`, or a text literal
    b = "\xFF"    #$ error[E0009]: `\x` above 0x7F is not valid in a text literal
                  #$ help: write the character, `\u{FF}`, or a byte string, `b"\xFF"`
    c = "\q"    #$ error[E0009]: unknown escape `\q`
    d = "\u{110000}"    #$ error[E0009]: 0x110000 is not a Unicode scalar value
    e = b"é"    #$ error[E0009]: a byte string may hold ASCII only
    f = f"}"    #$ error[E0009]: unmatched `}` in an f-string
    println(1)
