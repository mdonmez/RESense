# Linux Port Research

This is future-work research, not an implemented backend. Linux needs a native
hardware interface and state model separate from the Windows integration.

## Current Leads

The Linux-NitroSense work indicates that AN515-58 hardware is reachable
through an embedded-controller interface and a keyboard device/module. The
useful leads are:

| Surface | Lead |
| --- | --- |
| GPU fan mode | EC `0x21` |
| CPU fan mode | EC `0x22` |
| GPU manual speed | EC `0x3A` |
| CPU manual speed | EC `0x37` |
| CPU fan RPM | EC `0x13`, `0x14` |
| GPU fan RPM | EC `0x15`, `0x16` |
| CPU temperature | EC `0xB0` |
| GPU temperature | EC `0xB6` |
| System temperature | EC `0xB3` |
| Keyboard timeout | EC `0x06` |
| Operation mode | EC `0x2C` |
| Keyboard devices | `/dev/acer-gkbbl-0`, `/dev/acer-gkbbl-static-0` |

The observed mode values are quiet `0x00`, default `0x01`, and
extreme/performance `0x04`. These leads require validation on the target
machine; they are not production protocol guarantees.

## Expected Scope

Most realistic first targets:

- fan control and live RPM/temperature readback;
- operation mode control and readback;
- keyboard writes if the required kernel device exists.

Needs validation:

- exact per-fan mode and mixed auto/manual readback;
- complete keyboard live readback for effects, colors, brightness, and
  direction;
- LCD overdrive through EC, WMI, `acer-wmi`, sysfs, or another interface.

Windows-only features such as Sticky Keys, Windows/Menu lock, NitroSense
registry/XML synchronization, and WhisperMode are out of scope for the Linux
backend.

## Target Shape

Use separate platform modules for Linux EC controls and Linux keyboard-device
controls. Prefer live readback; where the hardware exposes only writes, store
RESense-managed state explicitly as persisted state.

Introduce shared abstractions only after a real Linux backend demonstrates a
stable common boundary.

## Reverse-Engineering Order

1. Confirm the machine with DMI and obtain read-only EC access.
2. Capture a baseline and compare one known feature change at a time.
3. Repeat reversible transitions such as `default -> quiet -> default` and
   `auto -> manual -> auto`.
4. Validate small manual fan changes, keyboard writes, and reboot behavior.
5. Require interface success, predicted state changes, physical behavior,
   idempotence, rollback, and understood reboot behavior before marking a
   feature supported.

Never fuzz unknown EC addresses. A live ISO also needs root access, the
required kernel modules, and suitable persistence; Secure Boot may block
out-of-tree modules.
