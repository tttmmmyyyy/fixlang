// Calls a function of type `void (*)(void*)` that a Fix program obtained, on `value`.
void call_refcount_funptr(void *funptr, void *value) {
    ((void (*)(void *))funptr)(value);
}
