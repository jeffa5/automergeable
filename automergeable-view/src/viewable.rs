use automerge::transaction::Transactable;
use automerge::transaction::Transaction;
use automerge::Automerge;
use automerge::AutomergeError;
use automerge::ChangeHash;
use automerge::ObjId;
use automerge::Prop;
use automerge::Value;

pub trait Viewable {
    fn keys(&self, obj: &ObjId) -> Vec<String>;

    fn keys_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> Vec<String>;

    fn length(&self, obj: &ObjId) -> usize;

    fn length_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> usize;

    fn value<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
    ) -> Result<Option<(Value, ObjId)>, AutomergeError>;

    fn value_at<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
        heads: &[ChangeHash],
    ) -> Result<Option<(Value, ObjId)>, AutomergeError>;
}

impl<'a> Viewable for Transaction<'a> {
    fn keys(&self, obj: &ObjId) -> Vec<String> {
        Transactable::keys(self, obj)
    }

    fn keys_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> Vec<String> {
        Transactable::keys_at(self, obj, heads)
    }

    fn length(&self, obj: &ObjId) -> usize {
        Transactable::length(self, obj)
    }

    fn length_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> usize {
        Transactable::length_at(self, obj, heads)
    }

    fn value<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
    ) -> Result<Option<(Value, ObjId)>, AutomergeError> {
        Transactable::value(self, obj, prop)
    }

    fn value_at<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
        heads: &[ChangeHash],
    ) -> Result<Option<(Value, ObjId)>, AutomergeError> {
        Transactable::value_at(self, obj, prop, heads)
    }
}

impl<'a> Viewable for Automerge {
    fn keys(&self, obj: &ObjId) -> Vec<String> {
        self.keys(obj)
    }

    fn keys_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> Vec<String> {
        self.keys_at(obj, heads)
    }

    fn length(&self, obj: &ObjId) -> usize {
        self.length(obj)
    }

    fn length_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> usize {
        self.length_at(obj, heads)
    }

    fn value<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
    ) -> Result<Option<(Value, ObjId)>, AutomergeError> {
        self.value(obj, prop)
    }

    fn value_at<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
        heads: &[ChangeHash],
    ) -> Result<Option<(Value, ObjId)>, AutomergeError> {
        self.value_at(obj, prop, heads)
    }
}
