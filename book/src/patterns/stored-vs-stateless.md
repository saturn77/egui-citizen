# Stored vs stateless panels

> **Stub.** The highest-value single chapter after the concepts —
> the one users hit hardest in practice.

Two lawful ways to use a citizen:

- **Stored:** panel is a field on the app struct, constructed once in
  `App::new()`, rendered via `self.panel.show(ui, ...)`. Use when the
  panel owns non-trivial local state (log buffer, image cache, terminal
  history).
- **Stateless per-frame:** panel is constructed fresh in the `TabKind`
  dispatch arm, e.g. `MyPanel::new(state).show(ui, &mut app)`. Use only
  when the panel's entire state comes from `app` / `services` and the
  citizen's reactive fields aren't being subscribed-to from elsewhere.

The trap, in detail: the stateless form *must* still receive a
`CitizenState` obtained from `dispatcher.register()`, **never** from
`CitizenState::default()`. Otherwise the panel and the dispatcher are
holding different `Arc`s and the reactive link is silently severed.

Cross-link: [the reactivity chapter](../concepts/state.md) covers the
shared-storage rule that makes this work.

Worked example: CopperForge's `tabs.rs` mixes both forms — `logger`,
`bom`, `terminal`, `shell`, `gerber_view_3d` are stored;
`DrcPanel`, `ViewSettingsPanel`, `SettingsPanel`, `ProjectsPanel` are
stateless. Walk through why each one chose what it did.
