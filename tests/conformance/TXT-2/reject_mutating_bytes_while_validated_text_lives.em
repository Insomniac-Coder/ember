#$ test: compile-fail
#$ rules: TXT-2, TXT-5, SPN-1
#$ error[E3021]: `bytes` is borrowed here
# A successful `to_str` view keeps its byte buffer borrowed.
fn main():
    bytes: Array[u8] = [104, 105]
    text = bytes.as_span().to_str()
    bytes.push(33)
    println(text)
