# TokenFuel visual QA

**Final result: passed** (local widget presentation and browser interactions). Authenticated provider integrations and physical device checks remain in `docs/office-validation.md`; this result does not certify them.

**Source and evidence**

- Source visual truth: `C:/WORK/TokenFuel/docs/design/selected-dock.png`, option 2 chosen by the user, 1448 × 1086 pixels.
- Implementation: `http://127.0.0.1:1420/?demo=1`, dark sample-data state, bars and rings. Source sample values reproduced (60%, 94%, 18%); countdowns advance naturally.
- Browser captures: `docs/design/implementation-bars.png`, `implementation-rings.png`; final widget crops `tokenfuel-bars.png` (832 × 259) and `tokenfuel-rings.png` (832 × 341). CSS viewport 860 × 760, devicePixelRatio 1. Source panels cropped to 1198 × 303 / 1198 × 387 and resized to the same 832-pixel width. Wallpaper/frame context excluded.
- Combined comparison input: `docs/design/comparison.png` contains source bars, implementation bars, source rings, implementation rings. Reviewed as one image. `scripts/compare-design.mjs` reproduces normalization.
- Additional evidence: `implementation-settings-light.png` and `implementation-narrow.png`. Narrow test 360 × 950; body scrollWidth 360, footer within viewport for both views.
- Screenshot API clipping produced blank images on one attempt. Those were rejected; final evidence uses verified full browser captures cropped to observed DOM bounds.

**Findings and comparison history**

1. [P2, fixed] Gauge hierarchy and thickness. Early rendering put percentages below bars and used thin rings. The selected concept puts percentages beside bars and uses a strong ring. Percentages moved beside rails, ring stroke increased to 10%, headings enlarged and header compacted. Combined normalized evidence confirmed the correction.
2. [P2, fixed] OpenAI ring countdown wrapped. The subsequent combined comparison showed a 116-pixel ring crowding the reset label. Ring diameter reduced to 100 pixels, reset copy kept on one line. Final combined capture shows all three reset labels readable.
3. [P2, fixed] Narrow bars. At 360 pixels the second provider retained a horizontal inset, and values in subsequent rows were above their rails. Responsive padding and value offsets corrected; revised narrow screenshot confirms aligned headings and values, with no horizontal overflow. Toggle width reduced to avoid crowding header tools.
4. [P2, fixed] Unknown values must not expose a zero-percent progress value to accessibility APIs. Disconnected cards show an empty decorative rail and a dash, without an asserted progress percentage.

**Required fidelity surfaces**

- Fonts/typography: system Segoe UI fallback, restrained 14-pixel brand and 17-pixel provider headings; small account/reset labels retain their hierarchy. No important reset wrapping in the final desktop capture. Source generated font cannot be identified precisely; system-native rendering is an accepted implementation choice.
- Spacing/layout: single rounded dock, equal provider columns and subtle separators; central Bars/Rings selector and right-aligned tools preserved. Native UI is slightly taller to retain source labels and explicit remaining context. Preview adds a sample-data disclosure row; it is absent in the Windows app. These are intentional information requirements, not framing drift.
- Colors/tokens: terracotta, pale teal and blue-violet provider accents deepen at remaining thresholds. Gemini retains its requested brand palette instead of the mock's pink warning; an explicit low-remaining label provides the warning. Light and opaque modes verified separately.
- Asset fidelity: real licensed Lobe provider SVGs; consistent Phosphor utility icons and fuel-pump brand mark. The licensed monochrome Gemini mark is tinted violet. Source Windows wallpaper is desktop context, not an app asset. No fabricated logos or decorative CSS illustrations.
- Copy/content: provider, account, quota, remaining and reset are present. Codex remains labelled Codex. Manual/experimental sources and sample preview are explicit. Ordinary ChatGPT/Gemini automatic and enterprise monthly verification limitations are available in settings/details.

The combined 832-pixel panel comparison makes headings, logos, values, reset labels and controls readable; a separate focused crop was not needed.

**Interactions and accessibility**

Verified bar/ring switching, quota expansion/closure, pin selection, theme selector, opaque setting, and disconnected first-run state. Settings content scrolls while the footer remains visible. Controls have accessible names, semantic buttons/inputs and focus styles; reduced-motion rules disable transitions/animations. Final fresh browser console showed no errors or warnings. Native drag, tray, startup, notifications, physical DPI and multi-monitor behaviour require the remaining device tests.

**Follow-up polish**

- [P3] Review type size and optical spacing on real Windows display scaling after office testing.
- [P3] Public-release brand-guideline review for provider marks.

**Implementation checklist**

- Selected dock and both views implemented.
- Recorded P2 findings corrected and recaptured.
- Light/opaque, narrow and empty states verified.
- Account-dependent/native validation recorded separately.

final result: passed
