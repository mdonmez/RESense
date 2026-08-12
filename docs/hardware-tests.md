# Hardware Tests

The hardware matrix is separate from code-level tests. It is feature-gated and ignored so `cargo test --all-targets` never writes to the laptop by accident.

## Run

Run from an interactive Windows session with NitroSense/PSSvc available:

```powershell
cargo test --features hardware-tests --test hardware_matrix -- --ignored --nocapture
```

The test uses the typed `Device` API and performs this sequence:

1. Capture the complete current state.
2. Run three verified cycles covering fan auto/max/custom, operation modes, keyboard lighting/brightness/timeout/Sticky Keys/Win/Menu lock, LCD Overdrive, and DTS sound presets when the current output supports them.
3. Restore the captured configuration and verify it.

The matrix waits one second after every mutation so vendor-service and Windows-session state have time to settle before the next operation or readback.

Live fan temperature and RPM are not compared during restoration because they naturally change. Fan mode and control percentages, keyboard state, operation mode, display state, and sound preset are compared.

Restoration runs even when a cycle returns an error and is retried during unwinding. This cannot protect against power loss, forced process termination, or Ctrl+C termination, so run the matrix only when you can leave the process running to completion. Sound and LCD overdrive checks run when applicable to the connected hardware.
