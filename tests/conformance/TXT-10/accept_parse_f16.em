#$ test: run-pass
#$ rules: TXT-10
#$ profiles: debug, release, shipping
#$ stdout: Ok(1.0) Ok(1.001) Ok(1.0) Ok(1.002)
#$ Ok(65500.0) Ok(inf) Ok(0.0) Ok(6e-08)
#$ Ok(0.1) Ok(-0.0) Ok(inf) Err(ParseError.Invalid) Err(ParseError.Empty)
# D-316 — `parse[f16]()` is the `f16` nearest the text, rounded once. Going
# through an `f64` rounds twice, which differs exactly where the `f64` lands
# on a midpoint between two `f16`s the text is just past (or short of).

fn main():
    println("1.00048828125".parse[f16](), "1.000488281250000000000000001".parse[f16](), "1.0004882812499999999999999".parse[f16](), "1.00146484375".parse[f16]())
    println("65519.99999999999999999".parse[f16](), "65520".parse[f16](), "2.98023223876953125e-8".parse[f16](), "2.98023223876953125000001e-8".parse[f16]())
    println("0.1".parse[f16](), "-0.0".parse[f16](), "inf".parse[f16](), "x".parse[f16](), "".parse[f16]())
