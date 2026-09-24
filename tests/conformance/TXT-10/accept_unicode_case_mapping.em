#$ test: run-pass
#$ rules: TXT-10
#$ stdout:
#$ HELLO, WÖRLD hello, wörld
#$ STRASSE οδος σας ǆ FI
#$ ADA LOVELACE 3
# `[TXT-10]` — `to_upper` and `to_lower` are Unicode-aware and depend on no
# locale: full case mapping (`ß` becomes `SS`, `ﬁ` becomes `FI`), and a
# capital sigma that ends a word lowers to `ς`, as Python's do.

println("Hello, Wörld".to_upper(), "Hello, Wörld".to_lower())
println("straße".to_upper(), "ΟΔΟΣ ΣΑΣ".to_lower(), "ǅ".to_lower(), "ﬁ".to_upper())
name: String = "Ada Lovelace"
println(name.to_upper(), name.to_lower().count("a"))
