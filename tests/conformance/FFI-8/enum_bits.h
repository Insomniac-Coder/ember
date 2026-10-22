/* The C side of accept_enum_bit_fields_are_read_and_written_by_accessors.em: bit-fields of a C
   enum, a signed one among them, beside an integer one. */
#ifndef ENUM_BITS_H
#define ENUM_BITS_H
enum Mode { MODE_OFF, MODE_ON, MODE_AUTO };
enum Bias { BIAS_LOW = -2, BIAS_NONE = 0, BIAS_HIGH = 1 };
struct Setting {
    enum Mode mode : 3;
    unsigned count : 6;
    enum Bias bias : 3;
};
static inline struct Setting setting_make(void) {
    struct Setting s = { MODE_AUTO, 33, BIAS_LOW };
    return s;
}
static inline int setting_mode(struct Setting s) { return (int)s.mode; }
static inline int setting_bias(struct Setting s) { return (int)s.bias; }
static inline unsigned setting_count(struct Setting s) { return s.count; }
#endif
