# egui-citizen — Task Findings

## egui_dock ui() is NOT a focus callback
In `egui_dock`, `ui()` runs for every *visible* tab across all dock nodes, not just
the focused one. Setting state inside `ui()` causes per-frame races when panels are
undocked. Use `on_tab_button` + `response.clicked()` for state transitions instead.

## activate() must be an encoded set/reset
When multiple algo panels are visible simultaneously, exactly one must be "active".
`CitizenRegistry::activate()` sets one citizen active and deactivates all others —
an encoded set/reset. This pattern is fundamental and cannot be achieved with
egui_dock's built-in focus tracking, which doesn't reliably report the last-clicked tab.

## HCR register values are always positive
The Stabiliti firmware stores all breakpoint values as positive unsigned integers.
Sign and mirroring are handled in the embedded processor, not in the register map.
The GUI must match this convention — store positive, mirror in the plot.
