# The problem

> **Stub.** Source material: `lib.rs:5-10` ("The problem" doc comment)
> and the per-frame race example below.

The premise: in `egui_dock`, every visible panel's `ui()` runs every
frame. If two panels both write to the same shared state, whichever
renders last wins — a per-frame race condition.

Worked example to write here: two panels, both holding `&mut
shared.flag`, last render wins. Lead into identity + dispatcher +
message-queue as the answer.
