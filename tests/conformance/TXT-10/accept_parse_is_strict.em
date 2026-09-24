#$ test: run-pass
#$ rules: TXT-10
#$ stdout:
#$ Ok(42) Ok(-7) Ok(3) Err(ParseError.Empty) Err(ParseError.Invalid) Err(ParseError.Invalid)
#$ Ok(255) Err(ParseError.Overflow) Err(ParseError.Invalid) Ok(-128) Err(ParseError.Overflow) Ok(9223372036854775807) Ok(-9223372036854775808) Err(ParseError.Overflow)
#$ Ok(2.5) Ok(-1000.0) Ok(0.5) Ok(5.0) Ok(inf) Ok(nan) Err(ParseError.Invalid) Err(ParseError.Invalid) Ok(0.1)
#$ Ok(true) Err(ParseError.Invalid) Ok('é') Err(ParseError.Invalid)
#$ 13
# `[TXT-10]` (ODR-029) — `parse[T]()` reads the whole text, skipping no white
# space: an integer is an optional sign (`-` only where signed) and digits that
# fit `T`; a float may be `inf`/`nan` or have a point and exponent; `bool` is
# `true` or `false`; `char` is one character. `ParseError` names the first
# problem: `Empty`, `Invalid` or `Overflow`.

println("42".parse[int](), "-7".parse[int](), "+3".parse[int](), "".parse[int](), "4x".parse[int](), " 1".parse[int]())
println("255".parse[u8](), "256".parse[u8](), "-1".parse[u8](), "-128".parse[i8](), "-129".parse[i8](), "9223372036854775807".parse[int](), "-9223372036854775808".parse[int](), "9223372036854775808".parse[int]())
println("2.5".parse[float](), "-1e3".parse[float](), ".5".parse[float](), "5.".parse[float](), "inf".parse[float](), "NaN".parse[float](), "1e".parse[float](), "e5".parse[float](), "0.1".parse[f32]())
println("true".parse[bool](), "True".parse[bool](), "é".parse[char](), "ab".parse[char]())
n = "12".parse[int]().unwrap_or(0)
println(n + 1)
