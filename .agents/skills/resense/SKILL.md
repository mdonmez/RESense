---
name: resense
description: 'Inspect and control supported Acer Nitro hardware through the RESense CLI: fans, operation modes, keyboard lighting, display overdrive, DTS sound, and system status. Use this skill whenever the user mentions RESense, NitroSense, PredatorSense, Acer Nitro hardware, fan control, keyboard lighting, or laptop operation modes.'
---

# RESense

Use RESense for supported Acer Nitro hardware control. It requires the Acer service and the default model check. Do not bypass that check unless the user explicitly asks for `--dangerously-allow-any-model`.

## Start And Updates

Before invoking any other RESense command in a task, run:

```powershell
resense --version
```

Record the installed version and any available update, but do not interrupt the user's request to discuss the update. Complete the requested RESense work first. After the work is complete, if an update was reported, tell the user that it is available and ask whether to run:

```powershell
resense update
```

Never update without the user's consent. If the user agrees, run the update after the requested work and then run `resense --version` again before continuing. If the update check is unavailable but the installed version is reported, complete the requested work and then state that the check could not be completed.

If `resense` is not available, offer the latest user-scoped installation:

```powershell
irm git.new/resense | iex
```

The interactive installer asks whether to install the optional RESense agent skill. For unattended skill installation or repair, use:

```powershell
& ([scriptblock]::Create((irm git.new/resense))) -YesSkill
```

Use `resense update` for normal binary updates. It never prompts about the skill. If the skill already exists, it refreshes it from the same release; if it does not exist, it remains absent.

Use a new terminal if the current shell has not refreshed its PATH.

## Command Discovery

Use `--help` whenever a command, subcommand, argument, or option is unclear. Check the narrowest relevant level instead of guessing:

```powershell
resense --help
resense fan --help
resense keyboard dynamic --help
```

Prefer the live help output over remembered syntax when the installed version or requested operation is unfamiliar.

## Read State

Use `status` for reads. Omit the target for the complete state or select one target for a focused read.

```powershell
resense status
resense status fan
resense status keyboard
resense status mode
resense status display
resense status sound
resense status --json
resense status --watch --interval 2
```

After a mutation, use the command's verified output or a focused status read to confirm the resulting state. Surface operational errors instead of guessing or substituting fallback values.

Fan status uses one selector and fixed blocks. `fan.mode` is the global behavior currently applied. Live CPU and GPU telemetry is under `fan.cpu` and `fan.gpu`. The saved custom settings are always under `fan.custom.cpu` and `fan.custom.gpu`, with a stable `mode` and `percent` field. When `fan.mode` is `auto` or `max`, do not describe a saved custom percentage as currently applied. When `fan.mode` is `custom`, the corresponding custom settings are applied. For example, `fan.mode=auto` with `fan.custom.cpu.mode=manual` and `fan.custom.cpu.percent=100` means the CPU is currently automatic and `100` is the saved custom percentage for a later custom-mode selection.

Keyboard lighting uses the same stable model. `keyboard.lighting.mode` selects the currently applied block. `keyboard.lighting.static` always contains the four static zones, and `keyboard.lighting.dynamic` always contains the saved dynamic effect settings. Do not move values between `active` and `stored` paths or describe the non-selected block as currently applied. Use the selector to interpret the fixed blocks.

## Fans

```powershell
resense fan auto
resense fan max
resense fan custom --cpu 70 --gpu-auto
```

`custom` accepts `--cpu <0-100>`, `--gpu <0-100>`, `--cpu-auto`, and `--gpu-auto`. At least one fan option is required, and a fan cannot receive both a percentage and its automatic flag.

## Operation Mode

```powershell
resense mode quiet
resense mode default
resense mode performance
resense mode performance --skip-whispermode
```

## Keyboard

```powershell
resense keyboard brightness 5
resense keyboard timeout disable
resense keyboard static --zone1 FF0000 --zone2 off
resense keyboard dynamic wave --speed 5 --color 00FFFF --direction from-left
resense keyboard sticky enable
resense keyboard win-menu disable
```

Keyboard lighting supports four static zones and dynamic effects including `breathing`, `neon`, `shifting`, `wave`, and `zoom`. Use `keyboard timeout` for the keyboard backlight timeout.

## Display

```powershell
resense display overdrive enable
resense display overdrive disable
```

## Sound

```powershell
resense sound auto
resense sound music
resense sound movies
resense sound voice
resense sound strategy
resense sound rpg
resense sound shooter
resense sound custom
```

Sound commands use the supported DTS path. The validated outputs are the internal speakers and wired 3.5 mm Realtek audio. Report unsupported output paths clearly instead of claiming that a preset was applied.

## Examples

Translate the user's intent into the smallest matching command, then verify the result when the operation changes hardware state.

```text
"Make my laptop quieter"        -> resense mode quiet
"I'm doing heavy work"          -> resense mode performance
"Reduce my keyboard brightness" -> resense keyboard brightness 2
"Cool down my laptop"           -> resense fan custom --cpu 80 --gpu 80
"Is my laptop overheating?"     -> resense status
"Set the keyboard to red"       -> resense keyboard static --zone1 FF0000
"Turn on LCD overdrive"         -> resense display overdrive enable
"Set the sound for music"       -> resense sound music
```
