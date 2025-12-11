pub trait Frame: Clone {
    fn empty(pc: usize) -> Self;
    fn set_is_user_mode(&mut self, is_user_mode: bool);
    fn is_user_mode(&self) -> bool;
    fn set_pc(&mut self, pc: usize);
    fn set_error(&mut self, args: (Option<usize>, Option<usize>, Option<usize>));
    fn set_success(&mut self, args: (Option<usize>, Option<usize>, Option<usize>));
}

