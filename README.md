# Takeable

> A simple wrapper type that holds values that need to be moved out from only a mutable reference.

## [Documentation](https://docs.rs/crates/takeable/0.1.0)

Sometimes it's useful to be able to move out of a mutable reference
temporarily or permanently. This often occurs for instance when dealing with state machines
implemented using an enum. For instance let's say you have the following state
machine:

```rust
enum State {
    Starting,
    Running(Resource1, Resource2),
    Finished(Option<Resource2>)
}
```

Let's say that you want to implement a function that changes state from
`Running` to `Finished`. The naive approach would be:

```rust
pub fn to_finished(state: &mut State) {
    let newstate = match *state {
        State::Starting => State::Finished(None),
        State::Running(_, r) => State::Finished(Some(r)),
        State::Finished(r) => State::Finished(r),
    };
    *state = newstate;
}
```

However, this would fail with a "cannot move out of borrowed content" error.

There are a few solutions to this problem:

- Use an `Option<State>`. Temporarily set it to `None` to move out the
  state.

- Introduce a new, invalid state for the same purpose.

- Use [`take`][take] from the [`take_mut`][take_mut] crate.

- Restructure your code to avoid the problem.

[take]: https://docs.rs/take_mut/latest/take_mut/fn.take.html
[take_mut]: https://crates.io/crates/take_mut

Depending on your scenario, any of these options might be preferable. This crate
provides a wrapper around an `Option<T>` with an API that forces correct usage
of the `Option`. This approach also has the advantage that it allows the
performance-optimization of not actually checking the enum-tag outside of
destructor-logic.

Using this library, the code could have been written like this:

```rust
struct StateMachine(Takeable<State>);

enum State {
    Starting,
    Running(Resource1, Resource2),
    Finished(Option<Resource2>)
}

pub fn to_finished(state: &mut StateMachine) {
    state.0.borrow(|state| {
        match state {
            State::Starting => State::Finished(None),
            State::Running(_, r) => State::Finished(Some(r)),
            State::Finished(r) => State::Finished(r),
        }
    });
}
```

It can also sometimes be useful to permanently move a value out while only
having a mutable reference. One such use case is when implementing `drop` and
needing to call a method of a field that consumes the field. This can be done
using this crate as follows:

```rust
struct Resource;
impl Resource {
    pub fn close(self) {}
}

struct ResourceUser {
    resource: Takeable<Resource>;
}
impl Drop for ResourceUser {
    fn drop(&mut self) {
        self.resource.take().close();
    }
}
```

The above code would also work by using an `Option` directly instead of a
`Takeable`. However, the latter has the advantage that it is clear by its
type that it must always have a value, and also that `None` variants do not
have to be handled when accessing the `Resource` elsewhere. Rather, the
`Takeable` will panic if this is attempted after the value has been moved
out.
