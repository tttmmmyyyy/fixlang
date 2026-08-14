// Calls the function the library built from `main.fix` exports, and reports through its exit status
// whether the answer is the one the Fix source gives.
//
// The library is named on the link line, which is how a C program that is built against a Fix
// library reaches it: the exported name has to be in the library's dynamic symbol table at link
// time as well as at run time.

// `Main::get_truth`, which `FFI_EXPORT` offers under this name.
int get_truth();

int main() {
    return get_truth() == 42 ? 0 : 1;
}
