# General
- [ ] The app should check if the platform is AN-515-58 model and if not, it should deny running. (Or run with `--dangerously-allow-any-model`)
- [ ] The entire CLI outputs should be rewritten to use the same style and to be more user-friendly and readable.
- [ ] The final re-lookup should be performed to ensure the app is written correctly as expected and wanted.
- [ ] Magic numbers should be replaced with named constants or configuration values.
- [ ] The error messages should be improved and more user-friendly.
- [ ] Update README and add docs/ for more details. Also add docs/reverse-engineering/ for more details on the reverse engineering process.
- [ ] The application attempts to connect to the PredatorSense pipe. If the background service is stopped, disabled, or not installed, it results in meaningless pipe errors. Before attempting the pipe connection, the application should verify whether the service is running and provide a clear warning to the user about starting the PredatorSense service.
- [ ] Improve code quality, readability and consistency.
- [ ] The original NitroSense app disables controlling the mode when running on AC, and disables fan control when on quiet mode. (I don't really remember this behavior - it will be reverse-engineered and verified how it works and will implement to resense correctly to behave like the original NitroSense app)

# New Feature Ideas
- [ ] Maybe a GUI with tray support
- [ ] Maybe custom fan curve control can be added with daemon like if >70° then fan speed = 100% etc. like
- [ ] Battery charge limit (feature at Acer Care Center) if it can be extractable from from NitroSense
- [ ] Custom preset profiles for fan, keyboard, and other settings like profile1 sets keyboard to static with fan speed at 100% etc. and easily switch between profiles.
  - [ ] App-based auto profile switching like when Cyberpunk.2077 is started, switch to profile2 and when it exits, switch back to profile1. etc.
- [ ] Nitro Key remapping to an new thing: like GUI (if created), mode switching, keyboard light switch etc. We have to think about more this.
- [ ] The feature `resense status --monitor --interval 5 --output log.csv` periodically (e.g., every 5 seconds) records CPU/GPU temperatures and fan RPM values into a CSV file, enabling post-session analysis of potential thermal throttling behavior.
- [ ] special stealth profile that makes keyboard white 2 level brightness, mode quiet, fans auto. (Like panic mode.)

## Fun Feature Ideas
- [ ] Audio Responsiveness: Make the keyboard lighting pulse in sync with the rhythm of the music playing on the computer by performing amplitude analysis. Like live audio visualization.
- [ ] MCP server that exposes the same functionality as the CLI, allowing LLMs to control the RESense remotely. (Auth required.) (Even LLMs can use CLI but this will offer a more standardized interface.)
- [ ] Transform the keyboard into a dynamic visual system that reacts to typing speed—idle stays in a cool tone (e.g., dark blue), increasing WPM shifts colors through green and yellow to flame red at peak speed, and when typing stops, the colors gradually cool back to blue, mirroring real-time activity.
- [ ] Map internet throughput to keyboard lighting speed by analyzing incoming (download) data—higher throughput accelerates “Wave” or “Shifting” effects; as a distinctive touch, exceeding 100 Mbps triggers a brief “Hyper” mode with white flashes, then transitions to a calm breathing effect once the download ends.
- [ ] Reflect the dominant on-screen color onto the keyboard by sampling the average color of pixels along the bottom of the display and mapping it to the keyboard’s four zones; as a distinctive touch, explosions shift the keyboard to orange while scenes like a forest turn it green, effectively extending the screen’s atmosphere to the keyboard.
- [ ] Monitor real-time load distribution between CPU and GPU using `resense keyboard visualizer --mode balance`, where Zones 1–2 represent CPU load (blue) and Zones 3–4 represent GPU load (green), with brightness scaling according to utilization so the more heavily used component appears more intense, enabling a direct visual read of system balance during rendering or gaming.
- [ ] Enable remote system control via a Telegram bot so RESense can receive commands (e.g., `/fan quiet`) to adjust settings like fan mode or query temperatures when you’re away from the computer, allowing actions such as silencing noisy fans or monitoring thermals directly from your phone.

# Status
- [ ] A --live flag should be added to the status command that shows the current state of the fan and CPU/GPU temperature in real-time.

# Keyboard

- [ ] When all four keyboard backlight zones are enabled, executing `.\resense.exe keyboard static --zone1 FFFFFF` currently disables zones 2, 3, and 4. The command must be scoped strictly to the specified zone(s), leaving other zones and overall brightness unchanged.
- [ ] When setting dynamic, if the speed, direction and color are not specified, the default values should be used. They should not be required, and the command should work with just the `dynamic mode` subcommand, with less friction.
- [ ] The brightness control should only be at `keyboard brightness`, the sub-brightness flags from dynamic and static should be removed and they should not touch brightness value.
- [ ] The direction values under dynamic should be renamed from left and right to fromleft and fromright to be more understandable.
