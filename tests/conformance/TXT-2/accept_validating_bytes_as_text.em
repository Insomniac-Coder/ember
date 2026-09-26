#$ test: run-pass
#$ rules: TXT-2, TXT-5
#$ profiles: debug, release, shipping
#$ stdout: Ok('Aéह😀') Err(Utf8Error.Invalid)
#$ stdout: Err(Utf8Error.Invalid) Err(Utf8Error.Invalid) Err(Utf8Error.Invalid)
#$ stdout: Ok('a\x00b')
# Converting a byte view to text validates UTF-8 without copying the bytes.
fn main():
    valid: Array[u8] = [65, 195, 169, 224, 164, 185, 240, 159, 152, 128]
    bad_lead: Array[u8] = [192, 175]
    truncated: Array[u8] = [226, 130]
    surrogate: Array[u8] = [237, 160, 128]
    too_high: Array[u8] = [244, 144, 128, 128]
    nul: Array[u8] = [97, 0, 98]
    v = valid.as_span()
    b = bad_lead.as_span()
    t = truncated.as_span()
    s = surrogate.as_span()
    h = too_high.as_span()
    n = nul.as_span()
    println(v.to_str(), b.to_str())
    println(t.to_str(), s.to_str(), h.to_str())
    println(n.to_str())
