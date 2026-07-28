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

int main(int argc, char **argv) {
    long n = atoll(argv[argc - 1]);
    Tree *t = make(n);
    long c = check(t);
    free_tree(t);
    printf("%ld\n", c);
    return 0;
}
