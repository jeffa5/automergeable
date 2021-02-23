pub trait AutoDiff {
    fn diff(&self, other: &Self) -> Vec<automerge::LocalChange> ;
}
