# Cheat sheet

> **Stub.** Single-page summary of the public API. For full signatures
> and doc comments, see [docs.rs/egui_citizen](https://docs.rs/egui_citizen).

## Dispatcher

| Call                                  | What it does                                                |
|---------------------------------------|-------------------------------------------------------------|
| `Dispatcher::new()`                   | Empty dispatcher.                                           |
| `register(id) -> CitizenState`        | Register a citizen, return a state handle wired in.         |
| `activate(&id)`                       | One-hot: this one on, all others off; emits messages.       |
| `send(message)`                       | Push a custom message onto the queue.                       |
| `drain_messages() -> Vec<...>`        | Take all pending messages. Call once per frame.             |
| `get(&id) -> Option<&CitizenState>`   | Look up a registered citizen's state.                       |
| `len() / is_empty()`                  | Count / emptiness.                                          |

## Citizen trait

| Method                            | Default? | Purpose                              |
|-----------------------------------|----------|--------------------------------------|
| `id() -> &CitizenId`              | required | Stable identity.                     |
| `state() -> &CitizenState`        | required | Read access to lifecycle state.      |
| `state_mut() -> &mut CitizenState`| required | Mutable access.                      |
| `on_activate()`                   | provided | Sets `state.active = true`.          |
| `on_deactivate()`                 | provided | Sets `state.active = false`.         |
| `on_click()`                      | provided | Sets `state.clicked = true`.         |
| `is_active() -> bool`             | provided | `state.active.get()`.                |
| `is_selected() -> bool`           | provided | `state.selected.get()`.              |

## CitizenState fields

`active`, `clicked`, `selected`, `moved`, `location`, `visible` — all
`Dynamic<T>`. See [Reactive lifecycle](concepts/state.md).

## CitizenMessage variants

`Activated { id }`, `Deactivated { id }`, `Clicked { id }`,
`Selected { id, selected }`, `Moved { id, location }`,
`VisibilityChanged { id, visible }`.
