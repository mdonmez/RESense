# General

- [x] The app now checks that the machine matches the AN515-58 family before running, and only bypasses that guard when `--dangerously-allow-any-model` is explicitly provided.
- [ ] The entire CLI outputs should be rewritten to use the same style and to be more user-friendly and readable.
- [ ] The final re-lookup should be performed to ensure the app is written correctly as expected and wanted.
- [ ] Magic numbers should be replaced with named constants or configuration values.
- [ ] The error messages should be improved and more user-friendly.
- [ ] Add broader docs/ for usage, safety, and implementation details.
- [x] Update README to reflect that NitroSense access returned on 2026-05-15 after KB5089549 while RESense development continues.
- [x] Before attempting PredatorSense pipe access, RESense now checks whether `PSSvc` (`Predator Service`) is installed and running, and returns a clear start/repair warning instead of raw pipe errors when it is unavailable.
- [ ] Improve code quality, readability and consistency.
- [x] Quiet mode now forces fan auto and blocks further fan control to keep the mode semantically quiet. RESense intentionally does not copy NitroSense's AC/Battery-Boost lock on operation mode control.
- [x] When brightness is changed with resense brightness command, the dynamic is overwritten and goes back to white static.
- [x] Add initial reverse-engineering notes for the restored NitroSense real-time comparison workflow.
- [x] Add a first non-invasive NitroSense GUI capture with process/package details, active pipes, UI Automation IDs, registry/XML snapshots, and `resense status` comparison.
- [x] Document that NitroSense GUI live-refresh attempts were reverted; RESense writes persisted state and NitroSense should be reopened when its GUI needs to show CLI changes.
- [x] Verify PredatorSense transport detection covers the lowercase service pipe name observed while NitroSense was running.
- [ ] More information about reverse engineering and named pipes will be gathered and documented as NitroSense can now be observed live again.
- [ ] Entire CLI skeleton should be rethought to be more user-friendly, understandable, simple and intuitive.

# Reverse Engineering

- [ ] Use restored NitroSense access as a live oracle: change one NitroSense UI setting at a time, then compare pipe traffic, HKLM state, ProgramData XML, hardware behavior, and `resense status`.
- [x] Complete fan read/write validation against NitroSense and remove unsupported ambiguity from the supported fan model.
- [x] Re-test keyboard zone live readback. With NitroSense open on 2026-05-16, UI/XML showed all zones on while service command `12` returned `1` for all zones; RESense had the decode inverted and now treats low byte `1` as enabled.
- [x] Confirm whether NitroSense dynamic lighting speed is actually `1..9`; UI Automation exposed `Speed_ScrollBar` range `1..9`, and RESense now accepts and documents `1..9` instead of the old `1..5` cap.
- [x] Re-check keyboard brightness, static/dynamic mode, and RGB color readback while NitroSense is open. Static and dynamic observations both tracked slider brightness through `LightingEffects.brightness`; `Key.brightness` was stale in both modes and should not be treated as the live slider source by itself.
- [x] Treat NitroSense GUI refresh as manual: close and reopen NitroSense after RESense CLI writes when the live interface needs to show the latest persisted values.
- [ ] Document any NitroSense UI-only policy behavior separately from RESense behavior before copying it. RESense should only copy policy restrictions when they are required for hardware safety or correctness.
- [ ] Keep `reverse-engineering/README.MD` updated with tested commands, observed payloads, state paths, and unresolved assumptions.

# New Feature Ideas

- [ ] Maybe a GUI with tray support
- [ ] Maybe custom fan curve control can be added with daemon like if >70° then fan speed = 100% etc. like
- [ ] Battery charge limit (feature at Acer Care Center) if it can be extractable from from NitroSense
- [ ] Custom preset profiles for fan, keyboard, and other settings like profile1 sets keyboard to static with fan speed at 100% etc. and easily switch between profiles.
  - [ ] App-based auto profile switching like when Cyberpunk.2077 is started, switch to profile2 and when it exits, switch back to profile1. etc.
- [ ] Nitro Key remapping to an new thing: like GUI (if created), mode switching, keyboard light switch etc. We have to think about more this.
- [ ] The feature `resense status --monitor --interval 5 --output log.csv` periodically (e.g., every 5 seconds) records CPU/GPU temperatures and fan RPM values into a CSV file, enabling post-session analysis of potential thermal throttling behavior.
- [ ] Special stealth profile that makes keyboard white 2 level brightness, mode quiet, fans auto. (Like panic mode.)

## Fun Feature Ideas

- [ ] Audio Responsiveness: Make the keyboard lighting pulse in sync with the rhythm of the music playing on the computer by performing amplitude analysis. Like live audio visualization.
- [ ] MCP server that exposes the same functionality as the CLI, allowing LLMs to control the RESense remotely. (Auth required.) (Even LLMs can use CLI but this will offer a more standardized interface.)
- [ ] Transform the keyboard into a dynamic visual system that reacts to typing speed—idle stays in a cool tone (e.g., dark blue), increasing WPM shifts colors through green and yellow to flame red at peak speed, and when typing stops, the colors gradually cool back to blue, mirroring real-time activity.
- [ ] Map internet throughput to keyboard lighting speed by analyzing incoming (download) data—higher throughput accelerates “Wave” or “Shifting” effects; as a distinctive touch, exceeding 100 Mbps triggers a brief “Hyper” mode with white flashes, then transitions to a calm breathing effect once the download ends.
- [ ] Reflect the dominant on-screen color onto the keyboard by sampling the average color of pixels along the bottom of the display and mapping it to the keyboard’s four zones; as a distinctive touch, explosions shift the keyboard to orange while scenes like a forest turn it green, effectively extending the screen’s atmosphere to the keyboard.
- [ ] Monitor real-time load distribution between CPU and GPU using `resense keyboard visualizer --mode balance`, where Zones 1–2 represent CPU load (blue) and Zones 3–4 represent GPU load (green), with brightness scaling according to utilization so the more heavily used component appears more intense, enabling a direct visual read of system balance during rendering or gaming.
- [ ] Enable remote system control via a Telegram bot so RESense can receive commands (e.g., `/fan quiet`) to adjust settings like fan mode or query temperatures when you’re away from the computer, allowing actions such as silencing noisy fans or monitoring thermals directly from your phone.

# Status

- [x] As of 2026-05-15, NitroSense is accessible again after KB5089549. RESense remains active as a programmable CLI and as a reverse-engineering target that can now be compared against the live NitroSense GUI.
- [ ] A --live flag should be added to the status command that shows the current state of the fan and CPU/GPU temperature in real-time.

# Keyboard

- [x] When all four keyboard backlight zones are enabled, executing `.\resense.exe keyboard static --zone1 FFFFFF` currently disables zones 2, 3, and 4. The command must be scoped strictly to the specified zone(s), leaving other zones and overall brightness unchanged.
- [x] When setting dynamic, missing speed, direction, and color are inferred from the closest current lighting state when possible, and only fall back to defaults when nothing usable exists. `resense keyboard dynamic <mode>` now works without requiring those flags.
- [x] The brightness control should only be at `keyboard brightness`, the sub-brightness flags from dynamic and static should be removed and they should not touch brightness value.
- [x] The direction values under dynamic should be renamed from left and right to fromleft and fromright to be more understandable.
