---
name: resense
description: 'Inspect and control supported Acer Nitro hardware through the RESense CLI: fans, operation modes, keyboard lighting, overdrive, DTS sound, and system status. Use this skill whenever the user mentions RESense, NitroSense, PredatorSense, Acer Nitro hardware, fan control, keyboard lighting, or laptop operation modes.'
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

Use `resense update` for normal binary updates. It waits for the update to finish in the same command and uses one exact stable GitHub Release. If the skill already exists, it refreshes it from that release; if it does not exist, it remains absent. A binary update succeeds before skill replacement is attempted, and a skill failure is reported explicitly.

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
resense status overdrive
resense status sound
resense status --json
resense status --watch --interval 2
```

Mutation commands print only the verified fields changed by that command. Use `resense status` or a focused status read when you need the complete state. Surface operational errors instead of guessing or substituting fallback values.

Fan status reports the behavior currently applied. `fan.mode` is `auto`, `max`, or `custom`, and live CPU and GPU telemetry is under `fan.cpu` and `fan.gpu`. `fan.settings.custom` appears only when `fan.mode=custom`. An automatic custom fan reports `mode=auto` without a percentage; a manual custom fan reports its current percentage. Never infer a manual override from an `auto` or `max` status.

Keyboard lighting reports only the currently applied effect. `keyboard.lighting.mode` is `static`, `breathing`, `neon`, `shifting`, `wave`, or `zoom`, followed by the matching `keyboard.lighting.settings.<mode>` fields. Static lighting reports four zones, `wave` reports speed and direction, `neon` reports speed, and other effects report only their supported fields. Do not invent inactive or unsupported fields.

## Fans

```powershell
resense fan auto
resense fan max
resense fan custom --cpu 70 --gpu-auto
```

`custom` requires one explicit selection for each fan. Use `--cpu <0-100>` or `--cpu-auto`, together with `--gpu <0-100>` or `--gpu-auto`. A fan cannot receive both a percentage and its automatic flag.

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
resense keyboard static --zone1 FF0000 --zone2 off --zone3 FF0000 --zone4 off
resense keyboard dynamic wave --speed 5 --direction from-left
resense keyboard sticky enable
resense keyboard win-menu disable
```

Keyboard lighting supports four static zones and dynamic effects including `breathing`, `neon`, `shifting`, `wave`, and `zoom`. Use `keyboard timeout` for the keyboard backlight timeout.

## Overdrive

```powershell
resense overdrive enable
resense overdrive disable
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
"Set the keyboard to red"       -> resense keyboard static --zone1 FF0000 --zone2 FF0000 --zone3 FF0000 --zone4 FF0000
"Turn on overdrive"             -> resense overdrive enable
"Set the sound for music"       -> resense sound music
```
