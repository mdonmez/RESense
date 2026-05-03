# General
- [ ] The app should check if the platform is AN-515-58 model and if not, it should deny running. (Or run with `--dangerously-allow-any-model`)
- [ ] The entire CLI outputs should be rewritten to use the same style and to be more user-friendly and readable.
- [ ] The final re-lookup should be performed to ensure the app is written correctly as expected and wanted.
- [ ] Magic numbers should be replaced with named constants or configuration values.
- [ ] The error messages should be improved and more user-friendly.
- [ ] Maybe a GUI with tray support?

# Status
- [ ] A --live flag should be added to the status command that shows the current state of the fan and CPU/GPU temperature in real-time.

# Keyboard

- [ ] When all four keyboard backlight zones are enabled, executing `.\resense.exe keyboard static --zone1 FFFFFF` currently disables zones 2, 3, and 4. The command must be scoped strictly to the specified zone(s), leaving other zones and overall brightness unchanged.
- [ ] When setting dynamic, if the speed, direction and color are not specified, the default values should be used. They should not be required, and the command should work with just the `dynamic mode` subcommand, with less friction.
- [ ] The brightness control should only be at `keyboard brightness`, the sub-brightness flags from dynamic and static should be removed and they should not touch brightness value.
- [ ] The direction values under dynamic should be renamed from left and right to fromleft and fromright to be more understandable.
