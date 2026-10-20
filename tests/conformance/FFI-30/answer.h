/* The C side of accept_one_c_function_declared_by_two_imports.em: a single-header library. */
#ifndef ANSWER_H
#define ANSWER_H
int answer_value(void);
#endif

#ifdef ANSWER_IMPLEMENTATION
int answer_value(void) { return 42; }
#endif
