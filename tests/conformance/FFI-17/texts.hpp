// The C++ side of the FFI-17 tests of contract words on functions Ember reaches through a wrapper
// (D-808): throwing functions and methods taking pointers and their lengths.
#pragma once
#include <cstddef>
#include <stdexcept>
#include <string>

// A throwing function: the bytes at `data`, summed.
inline int checksum(const unsigned char* data, std::size_t length) {
    if (length == 0) throw std::invalid_argument("no bytes");
    int sum = 0;
    for (std::size_t i = 0; i < length; ++i) sum += data[i];
    return sum;
}

// A throwing function whose result is a standard-library value: the bytes in hexadecimal.
inline std::string hex(const unsigned char* data, std::size_t length) {
    static const char digits[] = "0123456789abcdef";
    std::string out;
    for (std::size_t i = 0; i < length; ++i) {
        out += digits[data[i] >> 4];
        out += digits[data[i] & 15];
    }
    return out;
}

// A class by value, written where the caller says.
struct Ends {
    int first;
    int last;
};

inline Ends ends(const int* values, std::size_t count) {
    if (count == 0) throw std::out_of_range("no values");
    return Ends{values[0], values[count - 1]};
}

// A `T&` of a type by value beside a pointer and its length.
inline void scale(int& value, const int* factors, std::size_t count) {
    for (std::size_t i = 0; i < count; ++i) value *= factors[i];
}

// Two pointers sharing one length.
inline int dot(const int* left, const int* right, std::size_t count) {
    int sum = 0;
    for (std::size_t i = 0; i < count; ++i) sum += left[i] * right[i];
    return sum;
}

class Tally {
public:
    Tally() noexcept {}
    ~Tally() {}
    // A method that does not throw, reading a pointer and its length.
    void add(const int* values, std::size_t count) noexcept {
        for (std::size_t i = 0; i < count; ++i) total_ += values[i];
    }
    // A throwing `const` method, writing through one.
    int fill(int* out, std::size_t count) const {
        if (count > 4) throw std::length_error("too many slots");
        for (std::size_t i = 0; i < count; ++i) out[i] = total_ + static_cast<int>(i);
        return static_cast<int>(count);
    }
    int total() const noexcept { return total_; }
    // A pointer into the object.
    const int* spot() const noexcept { return &total_; }
    // A throwing method reading a C string.
    int parse(const char* text) {
        if (text[0] < '0' || text[0] > '9') throw std::invalid_argument("not a digit");
        total_ += text[0] - '0';
        return total_;
    }
    // A standard-library value lent mutably, beside bytes and their length.
    void append(std::string& text, const unsigned char* bytes, std::size_t count) const {
        text.append(reinterpret_cast<const char*>(bytes), count);
    }

private:
    int total_ = 0;
};

// Parameters named as the importer names what it passes itself.
inline Ends bounds(int result, int failed) {
    if (result > failed) throw std::invalid_argument("out of order");
    return Ends{result, failed};
}

// An opaque class by value, from a pointer and its length.
inline Tally tally_of(const int* values, std::size_t count) {
    Tally tally;
    tally.add(values, count);
    return tally;
}
