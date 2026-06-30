# RESense

_Reverse-engineered hardware control CLI for Acer Nitro AN515-58 systems._

---

## Why This Exists?

NitroSense’s interface stopped launching on the AN515-58, most likely after the 2026-04 Security Update (KB5083769) or the 2026-04 .NET Framework Security Update (KB5082417). At the time, even a full deep cleanup, uninstall, and reinstall did not restore access.

On 2026-05-15, after Windows update KB5089549, NitroSense became accessible again on this machine. That fixes the immediate lockout, but it does not remove the reason for RESense: core device controls should remain scriptable, inspectable, and usable even when the vendor GUI breaks.

The original outage was critical because it blocked access to basic device functions such as fan control and keyboard backlight control. Acer no longer appears to provide fresh NitroSense V3 updates for the Nitro 5 AN515-58, and the AN515-58 support page no longer exposes the download link for this software, even though it can still be downloaded from [this link](https://global-download.acer.com/GDFiles/Application/Nitro%20Sense/Nitro%20Sense_Acer_3.01.3052_20230420_W11x64_A.zip?acerid=638688149888116955&Step1=&Step2=&Step3=NITRO%20AN515-58&OS=ALL&LC=en&BC=ACER&SC=PA_6).

The purpose of RESense is to provide a small, programmable NitroSense-compatible control surface. Now that NitroSense works again, it also becomes a live reference application: settings can be changed in NitroSense while RESense observes named-pipe behavior, registry/XML state, and hardware readback in real time.

## AI Agent Skills

RESense is available as an [AI Agent Skill](https://agentskills.io/home). Add it to your agent environment with:

```bash
npx skills add https://github.com/mdonmez/RESense
```

This enables AI agents to query and control your Acer Nitro hardware directly. Learn more about AI Agent Skills at [agentskills.io](https://agentskills.io/home).

## Reverse Engineering Direction

RESense is still built around the NitroSense/PredatorSense service surface: named pipes, NitroSense-owned HKLM registry state, ProgramData lighting XML, and live hardware getters where they exist.

The restored NitroSense GUI opens a better verification path. Future reverse-engineering work should compare one NitroSense UI action at a time against:

- PredatorSense service/admin named-pipe requests and replies.
- NitroSense registry changes under `HKLM\SOFTWARE\OEM\NitroSense`.
- NitroSense ProgramData XML changes under `C:\ProgramData\OEM\NitroSense`.
- RESense command output and `resense status` readback.

See `reverse-engineering/README.MD` for the current workflow notes.

## Safety

RESense is intended for the Acer Nitro 5 AN515-58 family. By default it checks the detected system model before running and refuses to start on other machines. Use `--dangerously-allow-any-model` only if you explicitly want to bypass that safeguard.

RESense also checks that the NitroSense background service `PSSvc` (`Predator Service`) is installed and running before using the PredatorSense named pipes. If that service is stopped, disabled, or missing, RESense now fails with a clear warning instead of a low-level pipe error.
