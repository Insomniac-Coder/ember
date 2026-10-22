// The C++ side of the FFI-17a tests of views, optional values and `std::unique_ptr` parameters
// (C++20, for `std::span`).
#pragma once
#include <cstddef>
#include <memory>
#include <optional>
#include <span>
#include <stdexcept>
#include <string>
#include <string_view>

// Views as parameters.
inline std::size_t vowels(std::string_view text) noexcept {
    std::size_t count = 0;
    for (char c : text) count += (c == 'a' || c == 'e' || c == 'i' || c == 'o' || c == 'u');
    return count;
}
inline int sum(std::span<const int> values) noexcept {
    int total = 0;
    for (int value : values) total += value;
    return total;
}
inline void negate(std::span<int> values) noexcept {
    for (int& value : values) value = -value;
}
inline std::size_t shout_length(const std::string_view& text) {
    if (text.empty()) throw std::invalid_argument("nothing to shout");
    return text.size() + 1;
}

// Views as results: into the object, and into static storage.
class Words {
public:
    Words() : text_("ember burns bright"), scores_{3, 1, 2} {}
    std::string_view first() const noexcept { return std::string_view(text_).substr(0, text_.find(' ')); }
    std::span<const int> scores() const noexcept { return scores_; }
    std::span<int> scores_mut() noexcept { return scores_; }
    int total() const noexcept { return scores_[0] + scores_[1] + scores_[2]; }

private:
    std::string text_;
    int scores_[3];
};

inline std::string_view greeting() noexcept { return "hello"; }

// A view into the view it was given.
inline std::string_view trim(std::string_view text) noexcept {
    while (!text.empty() && text.front() == ' ') text.remove_prefix(1);
    while (!text.empty() && text.back() == ' ') text.remove_suffix(1);
    return text;
}

// Optional values.
inline std::optional<int> digit(int code) noexcept {
    if (code >= '0' && code <= '9') return code - '0';
    return std::nullopt;
}
inline int or_zero(std::optional<int> value) noexcept { return value.value_or(0); }
inline double scaled(const std::optional<double>& factor, double x) {
    if (x < 0) throw std::domain_error("negative");
    return factor ? *factor * x : x;
}

// A `std::unique_ptr` given to C++.
inline int destroyed_boxes = 0;
class Box {
public:
    explicit Box(int value) noexcept : value_(value) {}
    ~Box() { ++destroyed_boxes; }
    int value() const noexcept { return value_; }

private:
    int value_;
};

inline std::unique_ptr<Box> make_box(int value) { return std::make_unique<Box>(value); }
inline int consume(std::unique_ptr<Box> box) noexcept { return box->value() + 1; }
inline int boxes_destroyed() noexcept { return destroyed_boxes; }
