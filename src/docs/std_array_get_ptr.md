Get the pointer to the memory region where elements are stored.

This function is dangerous because if the array is not used after call of this function, the array will be deallocated soon and the returned pointer will be dangling.
Try using `borrow_ptr` instead.
