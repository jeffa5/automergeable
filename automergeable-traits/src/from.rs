/// Require a method to convert to a value from an automerge value.
pub trait FromAutomerge: Sized {
    type Error;

    fn from_automerge(value: &automerge::Value) -> Result<Self, Self::Error>;
}
