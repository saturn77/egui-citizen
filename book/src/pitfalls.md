# Common pitfalls

> **Stub.** Each item below needs a concrete broken snippet and the fix.

1. **Constructing `CitizenState` fresh per frame** severs reactivity.
   Use stored panels, or pass the dispatcher-registered state through
   to a stateless panel. See
   [Reactive lifecycle](concepts/state.md#the-trap-that-bites-everyone).
2. **Forgetting `drain_messages()`.** Messages accumulate in the queue
   forever; memory grows unbounded.
3. **Calling `activate()` every frame unconditionally.** Fires
   `Activated` / `Deactivated` messages every frame, floods the queue.
   Activate on the user-driven event (`response.clicked()`), not in the
   render path.
4. **Mixing panel-local state into `CitizenState`.** `CitizenState`
   has a fixed library-defined shape. Use panel struct fields. See
   [Where does state live?](patterns/state-shape.md).
5. **Expecting `visible` to track egui_dock's open/closed state
   automatically.** It doesn't. You must call `set()` on it yourself
   when a tab is closed or reopened, or route through
   `VisibilityChanged`.
6. **Two dispatchers in one app.** Don't. The one-hot invariant
   doesn't hold across dispatchers — `activate()` only deactivates
   citizens registered to the same dispatcher.
