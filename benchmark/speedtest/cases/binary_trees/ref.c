// The C counterpart of `main.fix`, on the same input, so the log can carry a
// reference the Fix line is read against. It checks the answer and prints nothing, as the
// Fix case does: a reference that computed something else would otherwise pass unnoticed.

#include <stdio.h>
#include <stdlib.h>

typedef struct Tree { struct Tree *l, *r; } Tree;

static Tree *make(long d) {
    Tree *t = malloc(sizeof(Tree));
    if (d == 0) { t->l = NULL; t->r = NULL; }
    else { t->l = make(d - 1); t->r = make(d - 1); }
    return t;
}

static long check(Tree *t) {
    return t->l ? 1 + check(t->l) + check(t->r) : 1;
}

static void free_tree(Tree *t) {
    if (t->l) { free_tree(t->l); free_tree(t->r); }
    free(t);
}

int main(void) {
    long n = 20;
    Tree *t = make(n);
    long c = check(t);
    free_tree(t);
    if (c != 2097151) { fprintf(stderr, "binary_trees: %ld\n", c); return 1; }
    return 0;
}
