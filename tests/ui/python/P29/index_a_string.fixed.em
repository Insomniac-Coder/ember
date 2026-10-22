fn main():
    word = "abc"
    letters = word.chars().collect[Array[char]]()
    println(letters[1])
