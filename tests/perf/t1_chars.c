#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char CHUNK[] = "the quick brown fox, jumps over\tthe lazy dog; h\xc3\xa9llo w\xc3\xb6rld\n";

static char* make_text(size_t* len) {
    size_t n = sizeof CHUNK - 1;
    char* text = (char*)malloc(n * 50000);
    for (int i = 0; i < 50000; i++) memcpy(text + (size_t)i * n, CHUNK, n);
    *len = n * 50000;
    return text;
}

/* One UTF-8 character at p (valid text): its code point; *width its bytes. */
static uint32_t decode(const unsigned char* p, int* width) {
    unsigned char b = p[0];
    if (b < 0x80) { *width = 1; return b; }
    if (b < 0xE0) { *width = 2; return ((uint32_t)(b & 0x1F) << 6) | (p[1] & 0x3F); }
    if (b < 0xF0) { *width = 3; return ((uint32_t)(b & 0x0F) << 12) | ((uint32_t)(p[1] & 0x3F) << 6) | (p[2] & 0x3F); }
    *width = 4;
    return ((uint32_t)(b & 0x07) << 18) | ((uint32_t)(p[1] & 0x3F) << 12) | ((uint32_t)(p[2] & 0x3F) << 6) | (p[3] & 0x3F);
}

static int is_space(uint32_t c) {
    return c == ' ' || (c >= 0x09 && c <= 0x0D) || (c >= 0x1C && c <= 0x1F) || c == 0x85 || c == 0xA0 || c == 0x1680
        || (c >= 0x2000 && c <= 0x200A) || c == 0x2028 || c == 0x2029 || c == 0x202F || c == 0x205F || c == 0x3000;
}

int main(void) {
    size_t len;
    const unsigned char* t = (const unsigned char*)make_text(&len);
    int64_t total = 0;
    for (int round = 0; round < 20; round++)
        for (size_t at = 0; at < len;) { int w; total += decode(t + at, &w); at += (size_t)w; }
    printf("%lld\n", (long long)total);
    return 0;
}
