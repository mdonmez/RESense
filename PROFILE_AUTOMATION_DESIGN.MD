# Profile and Automation Design Draft

This file records the current design discussion around profiles, conditions, rules, and automation in RESense.

Status:

- brainstorming only
- not yet implemented
- meant to guide later CLI and internal architecture decisions

## Core Conclusions

### 1. Profiles are state objects, not automation objects

A profile should mean:

- a named desired RESense hardware state
- based only on the validated supported surface
- applied through the same trusted write paths as normal commands

Profiles should support:

- snapshot-style creation from the current state
- programmatic creation/editing from scratch

But the default, lowest-friction path should be:

- save current state as a profile

So profiles are:

- reusable state snapshots/specs
- not trigger containers
- not daemon logic
- not raw registry/XML shortcut bundles

### 2. Conditions should not live inside profiles

“When X happens, do Y” should not be embedded into the profile itself.

Separation of concerns:

- profile = what state
- automation = when to run something
- daemon/monitor = what watches the system and executes automations

That separation keeps the system understandable and reusable.

### 3. `rule` should become a general automation layer

The idea evolved from:

- rules that apply profiles

to:

- automations that can trigger any validated RESense action

So the future automation layer should be able to:

- apply a profile
- run one direct command/action
- run a sequence of actions

Examples:

- on process start, apply `gaming`
- on battery, run `mode quiet`
- on headphones connect, run `sound music`
- on game exit, run `mode default` and `fan auto`

### 4. Automations should only use validated RESense actions

The automation layer should not start with arbitrary shell commands.

Instead, it should execute:

- validated RESense operations only

Benefits:

- safer
- easier to validate
- easier to keep compatible
- no second implementation path

Automation should reuse the same action engine as manual CLI usage.

### 5. Profiles should allow both full and partial state

Profiles should be able to represent:

- a full machine setup
- or only selected subsystems

That gives:

- full profiles like `gaming`, `night`, `work`
- partial profiles like keyboard-only themes or fan-only presets

Apply rule:

- missing field = leave current state unchanged
- present field = apply that state

## Profile Design

### Snapshot-first, spec-second

There are two real user intents:

#### A. Save what I have right now

Examples:

- “This setup is good, save it as `gaming`”
- “Save my current quiet keyboard/fan setup”

This should be the first-class UX:

```text
resense profile save gaming
```

#### B. Define a profile without first applying it manually

Examples:

- create a stealth profile from scratch
- generate profiles from a script
- build a profile in CI without interacting with the machine manually

This should also exist, but as a more explicit path.

Possible shapes later:

```text
resense profile init stealth
resense profile set stealth mode quiet
resense profile set stealth fan auto
```

or:

```text
resense profile import stealth.json
```

### Recommended internal model

Profiles should be declarative state documents.

Example:

```json
{
  "version": 1,
  "name": "gaming",
  "fan": {
    "mode": "custom",
    "cpu": { "auto": false, "percent": 70 },
    "gpu": { "auto": false, "percent": 80 }
  },
  "mode": {
    "value": "performance",
    "skip_whispermode": false
  },
  "keyboard": {
    "brightness": 5,
    "timeout": false,
    "lighting": {
      "kind": "dynamic",
      "mode": "wave",
      "speed": 7,
      "color": "#FF3300",
      "direction": "fromleft"
    },
    "sticky_keys": false,
    "win_menu_lock": true
  },
  "display": {
    "overdrive": true
  },
  "sound": {
    "preset": "shooter",
    "backend": "dts"
  }
}
```

### Supported profile surface

Profiles should include only validated supported state:

- fan
- mode
- keyboard
- display overdrive
- sound on the supported DTS/internal-speaker path

Not included for now:

- wired `3.5 mm` Waves path
- unsupported Bluetooth sound state
- anything outside the supported validated surface

### Apply semantics

Profiles should not bypass existing command logic.

Applying a profile should:

1. validate the profile fields
2. call the same trusted RESense write paths already used by normal commands
3. re-read the affected state
4. verify the resulting state matches the requested profile fields

### Save semantics

Needed first:

```text
resense profile save <name>
```

Possible later additions:

```text
resense profile save <name> --only keyboard,fan
resense profile show <name>
resense profile list
resense profile delete <name>
resense profile import <file>
resense profile export <name>
```

### Storage format

Preferred initial format:

- JSON

Reasons:

- easy to inspect
- easy to diff
- already natural for the repo/tooling
- no extra dependency pressure

### Storage location

Recommended first location:

- user-owned profile storage under the user profile / app data

Reason:

- profiles are a user-facing customization feature first

### Apply order

Recommended apply order:

1. mode
2. fan
3. keyboard
4. display
5. sound

Reason:

- mode can impose fan policy
- fan should be applied after mode
- the others are mostly independent

### Conflict handling

Example conflict:

- profile says `mode=quiet`
- profile also says custom manual fan values

Recommended rule:

- do not silently merge contradictory intent
- reject conflicting profile combinations clearly unless an explicit policy says otherwise

## Automation Design

### Better model

Automations should target actions, not just profiles.

Instead of:

- rule -> apply profile

use:

- automation -> run action list

That means a trigger can:

- apply a profile
- run one direct RESense action
- run multiple RESense actions in sequence

### Profiles still matter

Profiles remain valuable as reusable action bundles.

Examples:

- large coordinated state changes -> profiles
- small direct changes -> direct actions

### Do not start with arbitrary shell execution

Bad first version:

- arbitrary shell commands
- arbitrary scripts

Why avoid this first:

- too dangerous
- harder to validate
- harder to reason about

Better first version:

- actions may only invoke validated RESense operations

### Recommended action model

An automation should support actions like:

#### Apply a profile

```text
action: profile.apply("gaming")
```

#### Run a validated CLI-equivalent command

```text
action: mode.performance
action: fan.auto
action: sound.music
action: keyboard.brightness(2)
```

#### Run a sequence

```text
actions:
  - mode.quiet
  - keyboard.brightness(2)
  - sound.music
```

### Trigger tiers

#### Tier 1: high-value, realistic first triggers

- process start
- process stop
- AC plugged/unplugged
- headphones connected/disconnected
- session login

#### Tier 2: useful but more complex

- active foreground app changes
- battery percentage thresholds
- temperature thresholds
- GPU load thresholds

#### Tier 3: easy to overengineer

- complex nested condition trees
- arbitrary expression language
- heavy thermal rule engines

Do not start there.

### Conflict handling for automations

Hard question:

- two automations match at once
- which one wins?

Recommended first policy:

- only one active auto-applied state should win at a time
- newest matching automation wins
- when it deactivates, restore the most recent remaining matching automation or a fallback state

Do not try to merge multiple profiles early.

## Terminology Decision

Preferred naming:

- `profile`
- `automation`

Why prefer `automation` over `rule`:

- `rule` sounds narrower and lower-level
- `automation` better communicates trigger + actions

## Recommended Rollout Order

### Phase 1

Profiles only:

- save current state
- apply profile
- list/show/delete
- maybe import/export JSON

### Phase 2

Programmatic profile editing:

- create empty profile
- set profile fields from CLI
- validate structure

### Phase 3

Automation engine with first triggers:

- process start/stop
- AC power change
- headphones connect/disconnect

### Phase 4

More advanced automation behaviors if still needed.

## Likely Future CLI Direction

Not final, but conceptually:

```text
resense profile save gaming
resense profile apply gaming
resense profile list
resense profile show gaming
resense profile delete gaming
```

Possible later:

```text
resense profile save gaming --only keyboard,fan
resense profile set stealth mode quiet
resense profile import stealth.json
```

And for automations:

```text
resense automation add ...
resense automation list
resense automation show ...
resense automation remove ...
resense automation enable ...
resense automation disable ...
```

## Summary

The agreed model is:

- **profile** = desired state
- **automation** = trigger + actions
- **daemon/monitor** = runtime watcher/executor

Profiles should be:

- validated state snapshots/specs
- snapshot-first
- programmatically editable too
- partial or full
- applied through trusted write paths only

Automations should be:

- a general action layer, not just profile triggers
- limited to validated RESense operations first
- capable of running direct actions, profile applications, or ordered action sequences
