# Failure triage matrix

| Symptom | Likely cause | Inspect first | Bad workaround |
| --- | --- | --- | --- |
| Whole capture shifted | OS title bar, client crop, scale | screenshot bounds, content_rect, ppp | auto-align scoring images |
| All text is slightly different | font family/weight, antialiasing | glyph fallback, OS, renderer | apply blur to target |
| Sidebar edge displaced | panel width/margins | logical layout widths | move all child controls |
| Many missing sharp edges | missing widgets/icon assets | region manifest + actual tree | loosen edge threshold |
| Colors differ uniformly | theme, alpha compositing, color-space | palette, renderer, alpha matte | lower image opacity |
| One state is incorrect | wrong selected tab/scroll/modal | controlled app state | edit reference image |
| QA appears to improve but interactions fail | painted fake controls, lost state | kittest/accessibility tests | only evaluate screenshots |
| A later change breaks an earlier panel | shared layout tokens, root geometry | per-region history | ignore previously passing area |

## Iteration record

- reference hash, candidate build/revision, capture metadata
- region and metric before change
- predicted cause and minimal code edit
- metric after change, side effects
- keep/revert decision and remaining uncertainty

Set thresholds per project before optimization and do not tune to a failing result. When no edges exist in either image in a region, `edge_f1` is 1; inspect MAE and changed_fraction there instead.
