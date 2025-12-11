/// Aligns a given address up to the nearest specified alignment.
pub fn align_up(address: usize, alignment: usize) -> usize {
    let addr = address as usize;
    let mask = alignment - 1;
    if addr & mask == 0 {
        address
    } else {
        (addr + alignment) & !mask
    }
}

