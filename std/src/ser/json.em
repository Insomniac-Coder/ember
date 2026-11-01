## `std.ser.json` — `[SER-1]`'s JSON format: a struct is an object of its fields by name, with
## `"$version"` first when its version is above 0; an enum value is an object whose `"$variant"` names
## its variant, followed by its payload's fields; a list is an array, `None` is `null`.

from std.ser import Deserialize, Reader, SerError, Serialize, Writer

## Writes JSON text.
pub struct JsonWriter:
    out: String
    ## For each open object or array: whether it is an array, and whether nothing is in it yet.
    open: Array[(bool, bool)]

    ## It allocates a new `String`.
    pub static fn new() -> JsonWriter:
        return JsonWriter(String(), [])

    ## The text written.
    pub fn text() -> StrView:
        return this.out.as_str()

    ## A comma before an array's element after its first.
    ## It allocates as the `String` it writes to grows.
    fn before_value():
        n = len(this.open)
        if n > 0 and this.open[n - 1].0:
            if not this.open[n - 1].1:
                this.out.push_str(",")
            this.open[n - 1] = (true, false)

    ## It allocates as the `String` it writes to grows.
    fn quoted(text: StrView):
        this.out.push_str("\"")
        for c in text.chars():
            if c == '"':
                this.out.push_str("\\\"")
            elif c == '\\':
                this.out.push_str("\\\\")
            elif c == '\n':
                this.out.push_str("\\n")
            elif c == '\r':
                this.out.push_str("\\r")
            elif c == '\t':
                this.out.push_str("\\t")
            elif (c as u32) < 32:
                this.out.push_str(f"\\u{(c as u32):04x}")
            else:
                this.out.push(c)
        this.out.push_str("\"")

    ## It allocates as the `String` it writes to grows.
    fn key(name: StrView):
        n = len(this.open)
        if n > 0 and not this.open[n - 1].1:
            this.out.push_str(",")
        if n > 0:
            this.open[n - 1] = (false, false)
        this.quoted(name)
        this.out.push_str(":")

extend JsonWriter implements Writer:
    ## It allocates as the `String` it writes to grows.
    fn begin_struct(name: StrView, version: int) -> Result[void, SerError]:
        this.before_value()
        this.out.push_str("{")
        this.open.push(owns (false, true))
        if version > 0:
            this.key("$version")
            this.out.push_str(f"{version}")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn field(name: StrView) -> Result[void, SerError]:
        this.key(name)
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn end_struct() -> Result[void, SerError]:
        _ = this.open.pop()
        this.out.push_str("}")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn begin_variant(name: StrView) -> Result[void, SerError]:
        this.before_value()
        this.out.push_str("{")
        this.open.push(owns (false, true))
        this.key("$variant")
        this.quoted(name)
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn end_variant() -> Result[void, SerError]:
        return this.end_struct()

    ## It allocates as the `String` it writes to grows.
    fn begin_list(count: int) -> Result[void, SerError]:
        this.before_value()
        this.out.push_str("[")
        this.open.push(owns (true, true))
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn end_list() -> Result[void, SerError]:
        _ = this.open.pop()
        this.out.push_str("]")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn write_none() -> Result[void, SerError]:
        this.before_value()
        this.out.push_str("null")
        return Ok(())

    ## It allocates a new `String`.
    fn write_int(value: i64) -> Result[void, SerError]:
        this.before_value()
        this.out.push_str(f"{value}")
        return Ok(())

    ## It allocates a new `String`.
    fn write_float(value: f64) -> Result[void, SerError]:
        if value != value or value - value != 0.0:
            return Err(SerError.Malformed(String.from("JSON has no NaN or infinity")))
        this.before_value()
        this.out.push_str(f"{value}")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn write_bool(value: bool) -> Result[void, SerError]:
        this.before_value()
        this.out.push_str("true" if value else "false")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn write_str(value: StrView) -> Result[void, SerError]:
        this.before_value()
        this.quoted(value)
        return Ok(())

## Reads JSON text.
pub struct JsonReader:
    text: String
    at: int
    ## For each open array: whether no element has been read yet.
    lists: Array[bool]

    ## It allocates a new `String`.
    pub static fn new(text: StrView) -> JsonReader:
        return JsonReader(String.from(text), 0, [])

    ## It allocates the error's message when the input is malformed.
    fn malformed(what: StrView) -> SerError:
        return SerError.Malformed(f"{what} at byte {this.at}")

    fn peek() -> int:
        bytes = this.text.as_bytes()
        while this.at < len(bytes) and (bytes[this.at] == 32 or bytes[this.at] == 10 or bytes[this.at] == 13 or bytes[this.at] == 9):
            this.at += 1
        if this.at < len(bytes):
            return bytes[this.at] as int
        return -1

    ## It allocates the error's message when the input is malformed.
    fn expect(byte: int, what: StrView) -> Result[void, SerError]:
        if this.peek() != byte:
            return Err(this.malformed(f"expected {what}"))
        this.at += 1
        return Ok(())

    fn word(w: StrView) -> bool:
        _ = this.peek()
        if this.text.as_str()[this.at..].starts_with(w):
            this.at += w.as_bytes().len()
            return true
        return false

    ## It allocates a new `String`.
    fn string() -> Result[String, SerError]:
        this.expect(34, "a string")?
        out = String()
        bytes = this.text.as_bytes()
        start = this.at
        while this.at < len(bytes) and bytes[this.at] != 34:
            if bytes[this.at] == 92:
                out.push_str(this.text.as_str()[start..this.at])
                this.at += 1
                if this.at >= len(bytes):
                    break
                e = bytes[this.at]
                this.at += 1
                if e == 110:
                    out.push('\n')
                elif e == 116:
                    out.push('\t')
                elif e == 114:
                    out.push('\r')
                elif e == 117:
                    if this.at + 4 > len(bytes):
                        return Err(this.malformed("a short \\u escape"))
                    code: u32 = 0
                    for k in 0..4:
                        d = bytes[this.at + k] as u32
                        digit: u32 = 99
                        if d >= 48 and d <= 57:
                            digit = d - 48
                        elif d >= 97 and d <= 102:
                            digit = d - 87
                        elif d >= 65 and d <= 70:
                            digit = d - 55
                        if digit == 99:
                            return Err(this.malformed("a \\u escape"))
                        code = code * 16 + digit
                    this.at += 4
                    match char.from_u32(code):
                        Some(c):
                            out.push(c)
                        None:
                            return Err(this.malformed("a \\u escape"))
                else:
                    out.push(e as char)
                start = this.at
            else:
                this.at += 1
        if this.at >= len(bytes):
            return Err(this.malformed("an unterminated string"))
        out.push_str(this.text.as_str()[start..this.at])
        this.at += 1
        return Ok(out)

    fn number() -> StrView:
        _ = this.peek()
        bytes = this.text.as_bytes()
        start = this.at
        while this.at < len(bytes) and (bytes[this.at] == 45 or bytes[this.at] == 43 or bytes[this.at] == 46 or bytes[this.at] == 101 or bytes[this.at] == 69 or (bytes[this.at] >= 48 and bytes[this.at] <= 57)):
            this.at += 1
        return this.text.as_str()[start..this.at]

extend JsonReader implements Reader:
    ## It allocates the error's message when the input is malformed.
    fn begin_struct() -> Result[int, SerError]:
        this.expect(123, "`{`")?
        mark = this.at
        if this.peek() == 34:
            key = this.string()?
            if key.as_str() == "$version":
                this.expect(58, "`:`")?
                version = this.read_int()?
                if this.peek() == 44:
                    this.at += 1
                return Ok(version)
        this.at = mark
        return Ok(0)

    ## It allocates a new `String`.
    fn next_field() -> Result[Option[String], SerError]:
        c = this.peek()
        if c == 125:
            return Ok(None)
        if c == 44:
            this.at += 1
        name = this.string()?
        this.expect(58, "`:`")?
        return Ok(Some(name))

    ## It allocates the error's message when the input is malformed.
    fn end_struct() -> Result[void, SerError]:
        return this.expect(125, "`}`")

    ## It allocates the error's message when the input is malformed.
    fn begin_variant() -> Result[String, SerError]:
        this.expect(123, "`{`")?
        key = this.string()?
        if key.as_str() != "$variant":
            return Err(this.malformed("expected `\"$variant\"`"))
        this.expect(58, "`:`")?
        return this.string()

    ## It allocates the error's message when the input is malformed.
    fn end_variant() -> Result[void, SerError]:
        return this.expect(125, "`}`")

    ## It allocates as the `Array` it adds to grows.
    fn begin_list() -> Result[void, SerError]:
        this.expect(91, "`[`")?
        this.lists.push(owns true)
        return Ok(())

    ## It allocates the error's message when the input is malformed.
    fn next_element() -> Result[bool, SerError]:
        if this.peek() == 93:
            this.at += 1
            _ = this.lists.pop()
            return Ok(false)
        n = len(this.lists)
        if n > 0 and not this.lists[n - 1]:
            this.expect(44, "`,`")?
        if n > 0:
            this.lists[n - 1] = false
        return Ok(true)

    fn read_none() -> Result[bool, SerError]:
        return Ok(this.word("null"))

    ## It allocates the error's message when the input is malformed.
    fn read_int() -> Result[i64, SerError]:
        match this.number().parse[i64]():
            Ok(n):
                return Ok(n)
            Err(_):
                return Err(this.malformed("expected an integer"))

    ## It allocates the error's message when the input is malformed.
    fn read_float() -> Result[f64, SerError]:
        match this.number().parse[f64]():
            Ok(x):
                return Ok(x)
            Err(_):
                return Err(this.malformed("expected a number"))

    ## It allocates the error's message when the input is malformed.
    fn read_bool() -> Result[bool, SerError]:
        if this.word("true"):
            return Ok(true)
        if this.word("false"):
            return Ok(false)
        return Err(this.malformed("expected `true` or `false`"))

    ## It allocates a new `String`.
    fn read_str() -> Result[String, SerError]:
        return this.string()

    ## It allocates a new `String`.
    fn skip_value() -> Result[void, SerError]:
        c = this.peek()
        if c == 123:
            this.at += 1
            while this.peek() != 125:
                if this.peek() == 44:
                    this.at += 1
                _ = this.string()?
                this.expect(58, "`:`")?
                this.skip_value()?
            this.at += 1
            return Ok(())
        if c == 91:
            this.at += 1
            while this.peek() != 93:
                if this.peek() == 44:
                    this.at += 1
                this.skip_value()?
            this.at += 1
            return Ok(())
        if c == 34:
            _ = this.string()?
            return Ok(())
        if this.word("true") or this.word("false") or this.word("null"):
            return Ok(())
        if this.number().as_bytes().len() > 0:
            return Ok(())
        return Err(this.malformed("expected a value"))

## `value` as JSON text.
[T: needs Serialize]
pub fn to_string(value: T) -> Result[String, SerError]:
    w = JsonWriter.new()
    value.serialize(ref mut w)?
    return Ok(String.from(w.text()))

## A `T` read from JSON text.
[T: needs Deserialize]
pub fn from_str(text: StrView) -> Result[T, SerError]:
    r = JsonReader.new(text)
    return T.deserialize(ref mut r)
