# Feature & Module Reference Index

This directory documents each completed part of `egui-widgetkit`, explaining its architecture, how it integrates into consuming applications, and the customization parameters it makes available for configuration and settings pages.

---

## Completed Parts

| Document | Part | Status | Description |
|---|---|---|---|
| [`part_0_workspace_scaffolding.md`](file:///d:/randoms/egui-lib/docs/parts/part_0_workspace_scaffolding.md) | **Part 0** | ✅ Completed | Multi-crate virtual workspace, version pinning, crate isolation rules, and module conventions. |
| [`part_1_egui_layout.md`](file:///d:/randoms/egui-lib/docs/parts/part_1_egui_layout.md) | **Part 1** | ✅ Completed | Responsive nested splits, constraint relaxation solver, spacing engine, sub-pixel remainder absorption, and dynamic minimum window bounds. |
| [`part_2_spring_core.md`](file:///d:/randoms/egui-lib/docs/parts/part_2_spring_core.md) | **Part 2** | ✅ Completed | Pure math analytical closed-form ODE spring solver, harmonic regimes, presets, framerate independence, and two-fold settling criteria. |
| [`part_3_egui_spring.md`](file:///d:/randoms/egui-lib/docs/parts/part_3_egui_spring.md) | **Part 3** | ✅ Completed | 4-corner independent spring bundle, elastic smear selection highlight, Bézier-rounded boundary geometry ($\kappa = 0.55228$), and continuous motion repaint. |
| [`part_5_egui_themes.md`](file:///d:/randoms/egui-lib/docs/parts/part_5_egui_themes.md) | **Part 5** | ✅ Completed | Token-based theme engine, single source of truth palette, 12 presets, smooth exponential morphing, native egui visuals synchronization, and live component preview. |
| [`part_6_egui_nav_stack.md`](file:///d:/randoms/egui-lib/docs/parts/part_6_egui_nav_stack.md) | **Part 6** | ✅ Completed | App-owned back-stack screen navigation (`NavStack<K>`), Android Navigation 3 philosophy, safe post-render mutation, forward/backward data flow, and spring transitions. |

---

*Note: New documentation files will be added here as subsequent parts (Part 4 `egui-vim-nav` and Part 7 `test-app`) are completed.*
