/* The C side of accept_two_declarations_of_one_function_keep_their_contracts.em. */
#ifndef POINTERS_H
#define POINTERS_H
int pointers_read(const int* p);
int pointers_bump(int* p);
#endif
#ifdef POINTERS_IMPLEMENTATION
int pointers_read(const int* p) { return *p; }
int pointers_bump(int* p) { return ++*p; }
#endif
