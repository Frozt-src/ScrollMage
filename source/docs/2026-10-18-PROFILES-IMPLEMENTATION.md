# Profiles and scrolling feedback implementation plan

Historical implementation record, completed before GitHub publication. See the repository changelog for published versions.

Goal: implement the user's requested diagnostics, editable Default and fixed auto-applied presets, per-app tuning, accurate effect scope, and Windows Snap integration.

Architecture: retain the native Rust/Win32 interface and existing spring engine. Extend settings persistence without discarding legacy profiles; add native delivery diagnostics and safe hovered-window resolution; integrate native controls and tests. No external app code is imported.

Specification: the user requested all five audit improvements, comparison with SmoothScroll GitHub implementations, an editable Default profile, and fixed macOS style/Super smooth profiles that auto-apply.

Constraints: preserve personal files, published releases, exclusions, reverse/horizontal preferences, tagged-input loop prevention, modifier bypass and integrity checks. Do not claim that arbitrary apps expose elastic boundaries. New builds are staged for review; no GitHub publication or running-app replacement.

- [x] Settings: write failing persistence/migration tests; preserve legacy per-app overrides; remember editable Default separately; enforce fixed preset values on load and selection; roundtrip profiles; test invalid and duplicate profiles.
- [x] Runtime: retain short nonblocking hook; record accepted/bypassed/output counts and reasons; resolve hovered targets according to Windows wheel routing; never emit into a target that changed; expose a concise status snapshot; test policy formatting and routing decisions.
- [x] UI: write failing hidden-native tests; auto-save/apply preset selections transactionally; disable numeric edits for fixed presets; preserve saved Default when switching; label pending edits; add per-app profile editor; keep effect scope explicit.
- [x] Windows frame: return HTMAXBUTTON for the native maximize control and preserve its keyboard/click behavior; support useful narrow snap widths; add hit-test tests using Microsoft's published contract.
- [x] Research: inspect pinned source paths from Windows SmoothScroll implementations, distinguish browser-only projects, document gaps and app rendering limits with primary links and license boundaries.
- [x] Validation: run settings/engine/motion/layout and hidden-native tests, formatting and clippy; build static CRT release; inspect new render artifacts; run bounded live receiver only with desktop access; stage a separate executable with evidence.

Execution rulings: Default selection restores the user's stored editable tuning and applies it; subsequent edits use Apply. macOS style/Super smooth selections immediately save and activate canonical tuning without applying unrelated option drafts. Per-app profiles support editable distance/response or a fixed preset, plus removal back to global settings. Friction/elastic controls stay explicitly limited to Scroll Test because receiving-app boundaries are not exposed by ordinary wheel injection.

Ownership: settings/tests, native runtime/diagnostics, and UI/integration have separate writers. Review settings and runtime changes against this contract before integration, then review the complete change and validation evidence. Audit scratch files remain ignored.

Completed on a local branch. See [validation](VALIDATION-0.6.8.md) for the review artifact, passed checks and outstanding live acceptance checks. No release was published.
