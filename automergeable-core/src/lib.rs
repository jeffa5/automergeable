use std::collections::HashMap;
use core::hash::Hash;
use std::convert::TryInto;
use automerge::{LocalChange, Path, ScalarValue};

pub trait ToValue {
    fn to_value(&self) -> automerge::Value ;
}

pub trait AutoDiff : ToValue {
    fn diff(&self, path:Path, original: &Self) -> Vec<automerge::LocalChange> ;
}

impl<T> ToValue for Vec<T> where T:ToValue {
    fn to_value(&self) -> automerge::Value {
        let vals = self.iter().map(|v| v.to_value()).collect::<Vec<_>>();
        automerge::Value::Sequence(vals)
    }
}

impl<T> AutoDiff for Vec<T> where T : AutoDiff + PartialEq{
    fn diff(&self, path:Path,original: &Self) -> Vec<automerge::LocalChange>  {
        println!("diffing vec");
            let mut changes = Vec::new();
            for (i, n) in self.iter().enumerate() {
                if !original.iter().any(|o| o == n) {
                    changes.push(LocalChange::insert(path.clone().index(i.try_into().unwrap()), n.to_value()))
                }
            }
            changes
    }
}

impl<K,V> ToValue for HashMap<K, V> where K:ToValue, V:ToValue{

fn to_value(&self) -> automerge::Value { todo!() }
}

impl<K,V> AutoDiff for HashMap<K, V> where V:AutoDiff+PartialEq, K: ToValue+ToString+Eq+Hash{
    fn diff(&self, path:Path,original: &Self) -> Vec<automerge::LocalChange>  {
        println!("diffing hashmap");
        let mut changes = Vec::new();
        for  (k,v) in self.iter(){
            if let Some(o) = original.get(k)  {
                changes.append(&mut v.diff(path.clone().key(k.to_string()), o))
            } else {
                changes.push(LocalChange::insert(path.clone().key(k.to_string()), v.to_value()))
            }
        }
        changes
    }
}

impl ToValue for String {
    fn to_value(&self) -> automerge::Value  {
        ScalarValue::Str(self.to_owned()).into()
    }
}

impl AutoDiff for String {
    fn diff(&self, path: Path, original: &Self) -> Vec<automerge::LocalChange>  {
        println!("diffing string");
        vec![automerge::LocalChange::set(path, automerge::Value::Primitive(automerge::ScalarValue::Str(self.to_owned())))]
    }
}
