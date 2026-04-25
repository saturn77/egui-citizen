# The Dispatcher

> **Stub.** Source: `crates/egui_citizen/src/dispatcher.rs`.

The hub between the UI and the backend. Key calls:

- `register(id)` returns a `CitizenState` clone wired into the
  dispatcher's table. Hold onto it, hand it to the panel.
- `activate(&id)` is the encoded set/reset — one citizen live, all
  others off, atomically. Emits `Activated` and `Deactivated` messages.
- `drain_messages()` is the once-per-frame backend boundary.
- `send(message)` lets backend threads or app-level logic push their
  own messages onto the queue.

Cross-link to [the reactivity chapter](state.md) for why `register()`
matters and what breaks if you skip it.
