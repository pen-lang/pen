#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AsyncStackAction {
    Suspend,
    Resume,
    Restore,
}
