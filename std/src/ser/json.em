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
    pub fn new() -> JsonWriter:
        return JsonWriter(String(), [])

    ## The text written.
    pub fn text(self) -> str:
        return self.out.as_str()

    ## A comma before an array's element after its first.
    ## It allocates as the `String` it writes to grows.
    fn before_value(mut self):
        n = len(self.open)
        if n > 0 and self.open[n - 1].0:
            if not self.open[n - 1].1:
                self.out.push_str(",")
            self.open[n - 1] = (true, false)

    ## It allocates as the `String` it writes to grows.
    fn quoted(mut self, text: str):
        self.out.push_str("\"")
        for c in text.chars():
            if c == '"':
                self.out.push_str("\\\"")
            elif c == '\\':
                self.out.push_str("\\\\")
            elif c == '\n':
                self.out.push_str("\\n")
            elif c == '\r':
                self.out.push_str("\\r")
            elif c == '\t':
                self.out.push_str("\\t")
            elif (c as u32) < 32:
                self.out.push_str(f"\\u{(c as u32):04x}")
            else:
                self.out.push(c)
        self.out.push_str("\"")

    ## It allocates as the `String` it writes to grows.
    fn key(mut self, name: str):
        n = len(self.open)
        if n > 0 and not self.open[n - 1].1:
            self.out.push_str(",")
        if n > 0:
            self.open[n - 1] = (false, false)
        self.quoted(name)
        self.out.push_str(":")

extend JsonWriter implements Writer:
    ## It allocates as the `String` it writes to grows.
    fn begin_struct(mut self, name: str, version: int) -> Result[void, SerError]:
        self.before_value()
        self.out.push_str("{")
        self.open.push((false, true))
        if version > 0:
            self.key("$version")
            self.out.push_str(f"{version}")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn field(mut self, name: str) -> Result[void, SerError]:
        self.key(name)
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn end_struct(mut self) -> Result[void, SerError]:
        _ = self.open.pop()
        self.out.push_str("}")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn begin_variant(mut self, name: str) -> Result[void, SerError]:
        self.before_value()
        self.out.push_str("{")
        self.open.push((false, true))
        self.key("$variant")
        self.quoted(name)
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn end_variant(mut self) -> Result[void, SerError]:
        return self.end_struct()

    ## It allocates as the `String` it writes to grows.
    fn begin_list(mut self, count: int) -> Result[void, SerError]:
        self.before_value()
        self.out.push_str("[")
        self.open.push((true, true))
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn end_list(mut self) -> Result[void, SerError]:
        _ = self.open.pop()
        self.out.push_str("]")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn write_none(mut self) -> Result[void, SerError]:
        self.before_value()
        self.out.push_str("null")
        return Ok(())

    ## It allocates a new `String`.
    fn write_int(mut self, value: i64) -> Result[void, SerError]:
        self.before_value()
        self.out.push_str(f"{value}")
        return Ok(())

    ## It allocates a new `String`.
    fn write_float(mut self, value: f64) -> Result[void, SerError]:
        if value != value or value - value != 0.0:
            return Err(SerError.Malformed(String.from("JSON has no NaN or infinity")))
        self.before_value()
        self.out.push_str(f"{value}")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn write_bool(mut self, value: bool) -> Result[void, SerError]:
        self.before_value()
        self.out.push_str("true" if value else "false")
        return Ok(())

    ## It allocates as the `String` it writes to grows.
    fn write_str(mut self, value: str) -> Result[void, SerError]:
        self.before_value()
        self.quoted(value)
        return Ok(())

## Reads JSON text.
pub struct JsonReader:
    text: String
    at: int
    ## For each open array: whether no element has been read yet.
    lists: Array[bool]

    ## It allocates a new `String`.
    pub fn new(text: str) -> JsonReader:
        return JsonReader(String.from(text), 0, [])

    ## It allocates the error's message when the input is malformed.
    fn malformed(self, what: str) -> SerError:
        return SerError.Malformed(f"{what} at byte {self.at}")

    fn peek(mut self) -> int:
        bytes = self.text.as_bytes()
        while self.at < len(bytes) and (bytes[self.at] == 32 or bytes[self.at] == 10 or bytes[self.at] == 13 or bytes[self.at] == 9):
            self.at += 1
        if self.at < len(bytes):
            return bytes[self.at] as int
        return -1

    ## It allocates the error's message when the input is malformed.
    fn expect(mut self, byte: int, what: str) -> Result[void, SerError]:
        if self.peek() != byte:
            return Err(self.malformed(f"expected {what}"))
        self.at += 1
        return Ok(())

    fn word(mut self, w: str) -> bool:
        _ = self.peek()
        if self.text.as_str()[self.at..].starts_with(w):
            self.at += w.as_bytes().len()
            return true
        return false

    ## It allocates a new `String`.
    fn string(mut self) -> Result[String, SerError]:
        self.expect(34, "a string")?
        out = String()
        bytes = self.text.as_bytes()
        start = self.at
        while self.at < len(bytes) and bytes[self.at] != 34:
            if bytes[self.at] == 92:
                out.push_str(self.text.as_str()[start..self.at])
                self.at += 1
                if self.at >= len(bytes):
                    break
                e = bytes[self.at]
                self.at += 1
                if e == 110:
                    out.push('\n')
                elif e == 116:
                    out.push('\t')
                elif e == 114:
                    out.push('\r')
                elif e == 117:
                    if self.at + 4 > len(bytes):
                        return Err(self.malformed("a short \\u escape"))
                    code: u32 = 0
                    for k in 0..4:
                        d = bytes[self.at + k] as u32
                        digit: u32 = 99
                        if d >= 48 and d <= 57:
                            digit = d - 48
                        elif d >= 97 and d <= 102:
                            digit = d - 87
                        elif d >= 65 and d <= 70:
                            digit = d - 55
                        if digit == 99:
                            return Err(self.malformed("a \\u escape"))
                        code = code * 16 + digit
                    self.at += 4
                    match char.from_u32(code):
                        Some(c):
                            out.push(c)
                        None:
                            return Err(self.malformed("a \\u escape"))
                else:
                    out.push(e as char)
                start = self.at
            else:
                self.at += 1
        if self.at >= len(bytes):
            return Err(self.malformed("an unterminated string"))
        out.push_str(self.text.as_str()[start..self.at])
        self.at += 1
        return Ok(out)

    fn number(mut self) -> str:
        _ = self.peek()
        bytes = self.text.as_bytes()
        start = self.at
        while self.at < len(bytes) and (bytes[self.at] == 45 or bytes[self.at] == 43 or bytes[self.at] == 46 or bytes[self.at] == 101 or bytes[self.at] == 69 or (bytes[self.at] >= 48 and bytes[self.at] <= 57)):
            self.at += 1
        return self.text.as_str()[start..self.at]

extend JsonReader implements Reader:
    ## It allocates the error's message when the input is malformed.
    fn begin_struct(mut self) -> Result[int, SerError]:
        self.expect(123, "`{`")?
        mark = self.at
        if self.peek() == 34:
            key = self.string()?
            if key.as_str() == "$version":
                self.expect(58, "`:`")?
                version = self.read_int()?
                if self.peek() == 44:
                    self.at += 1
                return Ok(version)
        self.at = mark
        return Ok(0)

    ## It allocates a new `String`.
    fn next_field(mut self) -> Result[Option[String], SerError]:
        c = self.peek()
        if c == 125:
            return Ok(None)
        if c == 44:
            self.at += 1
        name = self.string()?
        self.expect(58, "`:`")?
        return Ok(Some(name))

    ## It allocates the error's message when the input is malformed.
    fn end_struct(mut self) -> Result[void, SerError]:
        return self.expect(125, "`}`")

    ## It allocates the error's message when the input is malformed.
    fn begin_variant(mut self) -> Result[String, SerError]:
        self.expect(123, "`{`")?
        key = self.string()?
        if key.as_str() != "$variant":
            return Err(self.malformed("expected `\"$variant\"`"))
        self.expect(58, "`:`")?
        return self.string()

    ## It allocates the error's message when the input is malformed.
    fn end_variant(mut self) -> Result[void, SerError]:
        return self.expect(125, "`}`")

    ## It allocates as the `Array` it adds to grows.
    fn begin_list(mut self) -> Result[void, SerError]:
        self.expect(91, "`[`")?
        self.lists.push(true)
        return Ok(())

    ## It allocates the error's message when the input is malformed.
    fn next_element(mut self) -> Result[bool, SerError]:
        if self.peek() == 93:
            self.at += 1
            _ = self.lists.pop()
            return Ok(false)
        n = len(self.lists)
        if n > 0 and not self.lists[n - 1]:
            self.expect(44, "`,`")?
        if n > 0:
            self.lists[n - 1] = false
        return Ok(true)

    fn read_none(mut self) -> Result[bool, SerError]:
        return Ok(self.word("null"))

    ## It allocates the error's message when the input is malformed.
    fn read_int(mut self) -> Result[i64, SerError]:
        match self.number().parse[i64]():
            Ok(n):
                return Ok(n)
            Err(_):
                return Err(self.malformed("expected an integer"))

    ## It allocates the error's message when the input is malformed.
    fn read_float(mut self) -> Result[f64, SerError]:
        match self.number().parse[f64]():
            Ok(x):
                return Ok(x)
            Err(_):
                return Err(self.malformed("expected a number"))

    ## It allocates the error's message when the input is malformed.
    fn read_bool(mut self) -> Result[bool, SerError]:
        if self.word("true"):
            return Ok(true)
        if self.word("false"):
            return Ok(false)
        return Err(self.malformed("expected `true` or `false`"))

    ## It allocates a new `String`.
    fn read_str(mut self) -> Result[String, SerError]:
        return self.string()

    ## It allocates a new `String`.
    fn skip_value(mut self) -> Result[void, SerError]:
        c = self.peek()
        if c == 123:
            self.at += 1
            while self.peek() != 125:
                if self.peek() == 44:
                    self.at += 1
                _ = self.string()?
                self.expect(58, "`:`")?
                self.skip_value()?
            self.at += 1
            return Ok(())
        if c == 91:
            self.at += 1
            while self.peek() != 93:
                if self.peek() == 44:
                    self.at += 1
                self.skip_value()?
            self.at += 1
            return Ok(())
        if c == 34:
            _ = self.string()?
            return Ok(())
        if self.word("true") or self.word("false") or self.word("null"):
            return Ok(())
        if self.number().as_bytes().len() > 0:
            return Ok(())
        return Err(self.malformed("expected a value"))

## `value` as JSON text.
pub fn to_string[T: Serialize](value: T) -> Result[String, SerError]:
    w = JsonWriter.new()
    value.serialize(w)?
    return Ok(String.from(w.text()))

## A `T` read from JSON text.
pub fn from_str[T: Deserialize](text: str) -> Result[T, SerError]:
    r = JsonReader.new(text)
    return T.deserialize(r)
