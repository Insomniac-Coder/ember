#$ test: run-pass
#$ rules: TXT-7
#$ stdout:
#$ 12 55357 56613
#$ true
#$ 1
#$ true 3
#$ 0

## `[TXT-7]` — text is UTF-8; `std.ffi.WideString` is UTF-16 for the APIs that need it, made and read
## back explicitly. A character above U+FFFF is a surrogate pair; an interior NUL is a `NulError`
## at its byte; units from C stop at the first 0, and an unpaired surrogate decodes as U+FFFD.

from std.ffi import WideString

fn main():
    text = "héllo, 世界 🔥"
    match text.to_wide():
        Ok(wide):
            units = wide.units()
            println(units.len(), units[10], units[11])
            println(wide.to_string() == text)
        Err(_):
            println("refused")
    match "a\0b".to_wide():
        Ok(_):
            println("accepted")
        Err(error):
            println(error.index)
    from_c: Array[u16] = [0x41, 0xD800, 0x42, 0, 0x43]
    back = WideString.from_units(from_c.as_span())
    println(back.to_string() == "A\u{FFFD}B", back.units().len())
    empty: Array[u16] = []
    println(WideString.from_units(empty.as_span()).units().len())
