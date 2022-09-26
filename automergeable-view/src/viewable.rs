use automerge::transaction::Transactable;
use automerge::transaction::Transaction;
use automerge::Automerge;
use automerge::AutomergeError;
use automerge::ChangeHash;
use automerge::Keys;
use automerge::KeysAt;
use automerge::ObjId;
use automerge::Prop;
use automerge::Value;

pub trait Viewable {
    fn keys(&self, obj: &ObjId) -> Keys;

    fn keys_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> KeysAt;

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

    fn text(&self, obj: &ObjId) -> Result<String, AutomergeError>;

    fn text_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> Result<String, AutomergeError>;
}

impl<'a> Viewable for Transaction<'a> {
    fn keys(&self, obj: &ObjId) -> Keys {
        Transactable::keys(self, obj)
    }

    fn keys_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> KeysAt {
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
        Transactable::get(self, obj, prop)
    }

    fn value_at<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
        heads: &[ChangeHash],
    ) -> Result<Option<(Value, ObjId)>, AutomergeError> {
        Transactable::get_at(self, obj, prop, heads)
    }

    fn text(&self, obj: &ObjId) -> Result<String, AutomergeError> {
        Transactable::text(self, obj)
    }

    fn text_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> Result<String, AutomergeError> {
        Transactable::text_at(self, obj, heads)
    }
}

impl Viewable for Automerge {
    fn keys(&self, obj: &ObjId) -> Keys {
        self.keys(obj)
    }

    fn keys_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> KeysAt {
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
        self.get(obj, prop)
    }

    fn value_at<P: Into<Prop>>(
        &self,
        obj: &ObjId,
        prop: P,
        heads: &[ChangeHash],
    ) -> Result<Option<(Value, ObjId)>, AutomergeError> {
        self.get_at(obj, prop, heads)
    }

    fn text(&self, obj: &ObjId) -> Result<String, AutomergeError> {
        self.text(obj)
    }

    fn text_at(&self, obj: &ObjId, heads: &[ChangeHash]) -> Result<String, AutomergeError> {
        self.text_at(obj, heads)
    }
}
