# Automergeable

Taking inspiration for a typed [automerge](https://github.com/automerge/automerge) from [here](https://github.com/automerge/automerge-rs/issues/22).

Derive functionality to convert to and from automerge Values.
Be able to perform diffs on these types.
Use a custom `Document` for your type to facilitate more natural interactions.

**Very alpha**

```rust
#[derive(Automergeable)]
struct A {
  #[automergeable(representation = "Text")]
  some_text: String,
  #[automergeable(representation = "Counter")]
  a_counter: i64,
  #[automergeable(representation = "Timestamp")]
  a_timestamp: i64,
  b: B,
}

#[derive(Automergeable)]
struct B {
    inner: u64,
}
```
