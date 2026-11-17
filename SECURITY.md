# Security

Gestouch 0.0.1 is experimental. It processes local touchpad events and can synthesize keystrokes in the current user's Windows session. Do not run untrusted binaries or imported configurations.

For private vulnerability reports, contact the eventual repository owner privately rather than posting reproduction steps in a public issue. (No contact address has been established yet.)

Security principles: no keylogging, no telemetry, no raw input network transmission, no app-side elevation, no arbitrary shell command actions, no elevated service IPC command execution, and no decoding untrusted HID buffers without bounds checking. A future service/agent IPC will require per-user ACLs, mutual identity verification and request allowlisting.

Third-party libraries must pass dependency and license review before production release.
