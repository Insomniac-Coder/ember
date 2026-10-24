// A25 returned when the views are of the parameter: accepted, tied to the caller text.
pub fn words_of(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}
