# CLI Design

This file is the working draft for the next RESense CLI redesign.

Status:

- implemented in the public CLI
- command names and hierarchy below are the current contract

## Design Goals

- Keep the CLI shallow.
- Keep command names direct and readable.
- Avoid redundant nouns and duplicate read commands at every subtree.
- Make the common path obvious for normal users.
- Keep machine-readable output available, but do not let it dominate the human-facing command shape.
- Match the validated hardware model we now actually support.

## Core Design Rule

Use one read-only command with an optional target:

- `status` with no target means the whole system
- `status <target>` means one subsystem
- direct state/action commands perform writes

This keeps reads separate from mutations without placing `status` beside
state values under every subsystem.

## Preferred Direction

```text
resense
├── status [fan|keyboard|mode|display|sound]
│   [--json] [--watch] [--interval <seconds>]
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
└── sound
    └── <auto|music|movies|voice|strategy|rpg|shooter|custom>
```

Developer reverse-engineering probes remain separate Cargo binaries. They are
not part of the normal `resense --help` command tree or the public CLI
compatibility contract.

## Why This Direction Is Better

### 1. `status` is the read-only entry point

`status` means:

- whole-machine overview when no target is supplied
- one subsystem when a target is supplied

Examples:

```text
resense status
resense status fan
resense status keyboard
resense status mode
resense status display
resense status sound
resense status --json
resense status --watch
```

There is no `get` command and no nested subsystem `status` command.

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

### Bad: force `status` under every subsystem

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

The lesson is to make the read operation explicit without adding ceremony.
For RESense, the clean mapping is:

- `status` -> whole-system or targeted state read
- direct commands -> mutations / state selection

## Settled Design Decisions

### Targeted reads

`status` accepts at most one target. There is no `status all`; omitting the
target already means all supported state.

### Developer tools

Developer probes are kept outside the public command tree and are excluded
from the public compatibility contract. They are used explicitly for
development or reverse-engineering work.

### Migration

The redesign is a clean move. Old command forms are removed rather than kept
as aliases, and the implementation does not promise backward compatibility
with the current CLI skeleton.

### Output shape

Successful output reports only the resulting state: what RESense read or
changed. It does not expose how the state was obtained.

Normal text output may use concise labels and sections. JSON output uses the
same state model for scripts. Neither successful output includes source paths,
pipe names, query numbers, registry keys, reliability labels, or explanatory
provenance fields.

Errors may still explain why an operation failed; provenance is not part of a
successful state response.

Example JSON shape:

```json
{
  "mode": "performance",
  "fan": {
    "mode": "custom",
    "custom": {
      "cpu": { "percent": 70, "auto": false },
      "gpu": { "percent": 50, "auto": true }
    }
  },
  "display": {
    "overdrive": true
  }
}
```

## Example Usage

```text
resense status
resense status --json
resense status --watch --interval 2

resense status fan
resense status keyboard --json
resense status mode

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
- `status` is the only public read command and may take one subsystem target.
- Developer probes are separate from the normal public command tree.
- The redesign is a clean break; old command forms are not retained as aliases.
- Successful text and JSON output report state only, without source/provenance metadata.
