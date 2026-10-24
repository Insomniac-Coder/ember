/* A25 attack workload: the word list built line by line, the usual Python file loop.
       all_words = []                               all_words: Array[str] = []
       for line in text.split("\n"):                for line in text.split("\n"):
           all_words.extend(line.lower().split())       all_words.extend(line.to_lower().split_whitespace())
   The text is the report's (../../containers/bench/a25/common.h: 32 MiB of words), with the space
   after every 72nd-or-later column turned into a newline (lines of about 72 to 85 bytes). Each line
   is lowered into its own new buffer (line.lower() makes a new string), split, and its words appended
   to one list; then every word is hashed (FNV-1a) and counted, and the list dropped. 5 passes.
   Under the answer each lowered line is a temporary that dies at the end of its iteration, while
   all_words lives on: views of many texts with different lifetimes, which the answer refuses. */
#include "../../containers/bench/a25/common.h"
static char *make_lines(size_t *len) {
    char *t = make_text(len);
    size_t col = 0;
    for (size_t i = 0; i < *len; i++) {
        if (t[i] == ' ' && col >= 72) { t[i] = '\n'; col = 0; } else col++;
    }
    return t;
}
/* for each line [s, e) of t: body */
#define FOR_LINES(t, tn, s, e, ...) \
    for (size_t s = 0, e; s < (tn); s = e + 1) { e = s; while (e < (tn) && (t)[e] != '\n') e++; __VA_ARGS__ }
#define FOR_WORDS(p, n, ws, we, ...) \
    for (size_t i_ = 0, ws, we; i_ < (n);) { while (i_ < (n) && (p)[i_] == ' ') i_++; ws = i_; \
        while (i_ < (n) && (p)[i_] != ' ') i_++; we = i_; if (we > ws) { __VA_ARGS__ } }
