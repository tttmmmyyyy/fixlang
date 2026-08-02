#include <string.h>
/* One `Array a` value is three words: the storage pointer, the size and the capacity. */
static char slot[24];
void steal_element(void *p) { memcpy(slot, p, 24); }
void plant_element(void *p) { memcpy(p, slot, 24); }
