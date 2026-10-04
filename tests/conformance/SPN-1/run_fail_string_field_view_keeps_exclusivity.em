#$ test: run-fail
#$ rules: SPN-1, EXC-19, EXC-15, EXC-1
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to TextBox.text while a read access to TextBox.text is active
# Root must verify exact diagnostic wording. No unsafe buffer access.

class TextBox:
    text: String

    fn init(mut self):
        self.text = String.from("valid UTF-8")

    fn clear_text(mut self):
        self.text.clear()

fn main():
    box = TextBox()
    alias = box
    view = box.text.as_str()
    alias.clear_text()
    println(view)
