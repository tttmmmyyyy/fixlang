// Bucket index over the program's globals, keyed by the top-level
// `TyCon` of the receiver position (the last curried argument). Used
// by score.rs to assign Tier 1/2/3 cheaply.
//
// Step 1 only contains the bare module declaration; the index lands in
// Step 2.
