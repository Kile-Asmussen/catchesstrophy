#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Mailbox<T: Clone + Copy>([T; 64]);
