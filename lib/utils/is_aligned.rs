pub fn is_aligned(ptr: usize, align: usize) -> bool {
    if align == 0 {
        return false; // Zero alignment is not valid
    }
    let mask = align - 1;
    (ptr as usize & mask) == 0
}
