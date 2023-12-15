use super::*;

// Borrowing optimization.
// Consider an application `f(x)`, where `f` is a function (pointer) which takes the ownership of `x` but releases it in its body.
// Assume that the expression `x` is a variable, i.e., it is not a temporary value.
// Furthermore, assume that the variable `x` is used after `f(x)`, i.e., `x` is not released after `f(x)`.
// Borrowing optimization replaces `f(x)` to an expression `f1(x1)`, where
// - `f1` is a function which does not release `x`.
// - `x1` is a `borrowed variable` expression for the variable `x`. When the expression `x1` is evaluated, it does not retain `x` even if `x` is used later.

// This function converts a name of a function to the name of a same function but it only borrows its argument (i.e., does not release the argument in its body).
// * `borrow_flag` - A flag value whose i-th bit is 1 if and only if the i-th argument of the function is borrowed.
pub fn convert_to_borrowing_function_name(name: &mut FullName, borrower_flag: u8) {
    let name = name.name_as_mut();
}

// Perform borrowing optimization.
pub fn borrowing_optimization(program: &mut Program) {
    define_borrowing_function(program);
    todo!("")
}

// Adds a borrowing version of functions in a program.
pub fn define_borrowing_function(program: &mut Program) {
    // Currently, we only define several borrowing functions by hand.

    // Define borrowing version of `Std::Array:@`.
    let mut name = array_getter_function_name();
    // Get the name of the uncurried version.
    convert_to_funptr_name(name.name_as_mut(), 2);
}
