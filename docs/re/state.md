# RESense Source-of-Truth Matrix

| Feature | Read source | Write source | Verification | Status |
| --- | --- | --- | --- | --- |
| Fan RPM/temperature | live service `cmd 13` | none | compared with NitroSense telemetry | validated |
| Fan mode and remembered percentages | HKLM NitroSense `FanControl` plus live telemetry | service `cmd 15/16` plus HKLM mirror | mode/control readback | validated |
| Keyboard lighting | installed NitroSense `Main.xml` | service `cmd 27/28/29` plus atomic XML replacement | parse XML after write and read resulting state | validated persisted source |
| Keyboard timeout | live service `cmd 20` | service `cmd 17` | live readback | validated |
| Sticky Keys | current-session Windows API | current-session admin `cmd 2` plus HKLM mirror | current-session Windows readback | validated session-scoped |
| Windows/Menu lock | service `cmd 10/query 0` | service `cmd 9` plus HKLM mirror | live readback | validated |
| Operation mode | service `cmd 34/query 11` | service `cmd 30`, optional WhisperMode, HKLM mirror | live mode readback | validated |
| LCD overdrive | support flag plus service `cmd 10/query 0` | service `cmd 9` | live readback | validated |
| DTS sound preset | shared/admin `cmd 13` gated by `DTS_Audio_Support` | shared/admin `cmd 14` | shared/admin readback | validated internal and wired 3.5 mm |

## Deliberate Boundaries

- NitroSense XML is authoritative for the supported keyboard surface because no independent live getter was validated.
- `cmd 12` is not used for production keyboard zone state.
- Unknown protocol codes are errors, not `unknown`, `partial`, or `-like` production states.
- Bluetooth and other non-DTS sound paths are unsupported.
- Sticky Keys targets the current Windows session only. Sound and WhisperMode use reachable admin-agent sessions because their controlled state is shared on the validated machine.

## Assumptions And Resolutions

This section records remaining open protocol questions and the conclusions
that replaced earlier assumptions. Open items are outside the supported
surface; they must not become fabricated production states.

### Open

- The exact meaning of successful `cmd 20` queries outside the supported
  keyboard backlight-timeout decode is unknown.
- The exact meaning of successful `cmd 34` queries outside operation-mode
  query `11` is unknown.
- No independent live keyboard-state getter has been validated in the known
  service/admin command families.

### Resolved

- `cmd 30` return codes are non-authoritative; mode writes are verified with
  `cmd 34/query 11`.
- `cmd 10/query 512` is diagnostic only; fan mode comes from the validated
  combined live/registry model.
- `cmd 12` is not a trustworthy per-zone keyboard getter on this machine;
  keyboard lighting remains XML-backed.
- Sticky Keys is session-scoped, while DTS sound and WhisperMode are
  effectively shared/global on the validated machine.
- The wired 3.5 mm Realtek output uses the same validated DTS command surface
  as internal speakers; Bluetooth/non-DTS remains unsupported.
