# CLI Design Draft

This file is the working draft for the next RESense CLI redesign.

Status:

- brainstorming only
- not yet implemented
- command names and hierarchy are still open to change

## Design Goals

- Keep the CLI shallow.
- Keep command names direct and readable.
- Avoid redundant nouns and duplicate read commands at every subtree.
- Make the common path obvious for normal users.
- Keep machine-readable output available, but do not let it dominate the human-facing command shape.
- Match the validated hardware model we now actually support.

## Core Design Rule

Do not force one read verb everywhere.

Instead:

- use `status` for whole-system overview
- use `get` for subsystem-specific reads
- use direct state/action commands for writes

This avoids awkward trees like:

```text
resense sound status
resense sound music
```

where `status` sits beside actual state values as if it were one of them.

## Preferred Direction

```text
resense
├── status [--json] [--watch] [--interval <seconds>]
├── get {fan|keyboard|mode|display|sound} [--json]
├── fan
│   ├── auto
│   ├── max
│   └── custom [--cpu <percent>|--cpu-auto] [--gpu <percent>|--gpu-auto]
├── mode {quiet|default|performance} [--skip-whispermode]
├── keyboard
│   ├── brightness <1-5>
│   ├── timeout {enable|disable}
│   ├── static [--zone1 <hex|off>] [--zone2 <hex|off>] [--zone3 <hex|off>] [--zone4 <hex|off>]
│   ├── dynamic <breathing|neon|shifting|wave|zoom>
│   │   [--speed <1-9>]
│   │   [--color <hex>]
│   │   [--direction <fromleft|fromright>]
│   ├── sticky {enable|disable}
│   └── win-menu {enable|disable}
├── display
│   └── overdrive {enable|disable}
├── sound
│   └── <auto|music|movies|voice|strategy|rpg|shooter|custom> [--backend <auto|dts>]
└── debug
    └── ...
```

## Why This Direction Is Better

### 1. `status` is now one thing

`status` means:

- whole-machine overview

Examples:

```text
resense status
resense status --json
resense status --watch
```

It is no longer scattered under every subsystem.

### 2. `get` is now one thing

`get` means:

- read one subsystem in detail

Examples:

```text
resense get fan
resense get keyboard
resense get mode
resense get display
resense get sound
```

This is cleaner than mixing:

- `fan status`
- `sound status`
- no `mode status`

### 3. Write commands stay short and natural

Examples:

```text
resense fan auto
resense fan max
resense fan custom --cpu 70 --gpu-auto
resense mode performance
resense keyboard brightness 5
resense keyboard timeout enable
resense keyboard static --zone1 FF0000 --zone2 off
resense keyboard dynamic wave --speed 5 --direction fromleft
resense sound music
resense display overdrive enable
```

These read like intent, not schema.

### 4. Depth stays low

Most common commands stay at:

- subsystem + action/value

instead of:

- subsystem + nested noun + action + value

## Comparison With Worse Alternatives

### Bad: duplicate nested read commands

```text
resense sound status
resense sound music
resense mode performance
```

Problem:

- `status` looks like just another child command beside actual state values
- `mode` becomes asymmetrical for no good reason

### Bad: force `status` everywhere

```text
resense mode status
resense mode performance
```

Problem:

- extra depth
- more typing
- no real gain
- less natural than `resense mode performance`

### Too heavy: everything becomes `set/get`

```text
resense mode get
resense mode set performance
resense sound get
resense sound set music
```

Problem:

- consistent, but bureaucratic
- worse for a hardware CLI that should feel direct

## Why This Fits Enterprise-Grade CLI Patterns

Good CLIs do not force one universal read word everywhere.

They use different patterns depending on the shape of the thing:

- `git status`
- `kubectl get pods`
- `aws ec2 describe-instances`
- `systemctl status ssh`

The lesson:

- overview reads, resource reads, and mutating actions should not all be forced into one structure

For RESense, the clean mapping is:

- `status` -> overview
- `get` -> subsystem read
- direct commands -> mutations / state selection

## Active Open Questions

### 1. Should `get` support multiple targets?

Possible:

```text
resense get fan keyboard mode
```

Or keep it simple:

```text
resense get fan
```

Current leaning:

- keep `get` single-target

### 2. Should `get` also support `all`?

Possible:

```text
resense get all
```

But this may just duplicate `status`.

Current leaning:

- do not add `get all`
- keep `status` as the overview read

### 3. Should `debug` stay user-visible?

Likely yes, but clearly separated from the supported normal UX.

Current leaning:

- keep `debug`
- keep all probes and reverse-engineering helpers there

## Example Usage

```text
resense status
resense status --json
resense status --watch --interval 2

resense get fan
resense get keyboard --json
resense get mode

resense fan auto
resense fan max
resense fan custom --cpu 70 --gpu-auto

resense mode quiet
resense mode default
resense mode performance

resense keyboard brightness 5
resense keyboard timeout enable
resense keyboard static --zone1 FF0000 --zone2 off --zone3 00FF00 --zone4 off
resense keyboard dynamic breathing --speed 9 --color 00FFFF
resense keyboard sticky enable
resense keyboard win-menu disable

resense display overdrive enable

resense sound auto
resense sound music
resense sound shooter
```

## Notes

- `fan custom` is preferred over `fan set`.
- `sound` should avoid a `set` prefix and use direct preset names.
- `status` should not be nested under subsystem parents in this design.
- `--json` should remain available for scripting, but normal text output should be redesigned separately.
