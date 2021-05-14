# Automergeable

Taking inspiration for a typed [automerge](https://github.com/automerge/automerge) from [here](https://github.com/automerge/automerge-rs/issues/22).

- Derive functionality to convert to and from automerge `Value`s.
- Be able to perform diffs on these types.
- Use a custom `Document` for your type to facilitate more natural interactions.

**Very alpha**

```rust
#[derive(Automergeable)]
struct A {
  #[automergeable(representation = "text")]
  some_text: String,
  #[automergeable(representation = "counter")]
  a_counter: i64,
  #[automergeable(representation = "timestamp")]
  a_timestamp: i64,
  b: B,
}

#[derive(Automergeable)]
struct B {
    inner: u64,
}
```

## Fuzzing

Run fuzzing tests from the `automergeable` directory with `cargo fuzz run fuzz_target_1`.
