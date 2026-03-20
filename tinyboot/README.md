# tinyboot

Part of the [tinyboot](https://github.com/OpenServoCore/tinyboot) project — see the main README to get started.

Platform-agnostic bootloader core: protocol dispatcher, boot state machine, and app validation.

## Boot State Machine

Three states, encoded as contiguous 1-bit runs for cheap forward transitions (1→0 bit clear):

```
Idle (0xFF) → Updating (0x7F) → Validating (0x3F) → Idle (0xFF)
```

### State Transition Table

| Operation | Current State | Next State | Gate | Persistence | How |
|-----------|--------------|------------|------|-------------|-----|
| **Erase** | Idle | Updating | addr/size valid | step down state byte | Normal start of firmware update |
| **Erase** | Updating | Updating | addr/size valid | none | Subsequent erase pages during update |
| **Erase** | Validating | Updating | addr/size valid | refresh (state=Updating, clear checksum) | App failed to confirm, reflashing |
| **Write** | Idle | reject | | | Bug in host tool, no erase first |
| **Write** | Updating | Updating | addr/size valid | none | Normal firmware write during update |
| **Write** | Validating | reject | | | Bug in host tool |
| **Verify** | Idle | reject | | | Bug in host tool, no erase/write first |
| **Verify** | Updating | Validating | CRC match | refresh (state=Validating, write checksum) | Normal end of firmware write |
| **Verify** | Validating | reject | | | Bug in host tool, double verify |
| **Confirm** | Idle | Idle | | none | App confirms after already confirmed, harmless |
| **Confirm** | Updating | reject | | | Bug in app, update in progress |
| **Confirm** | Validating | Idle | app is alive | refresh (state=Idle, preserve checksum) | Normal app startup, confirms boot |
| **Boot app** | Idle | (boot) | validate_app passes | none | Normal power-on, app is valid |
| **Boot app** | Updating | (bootloader) | | none | Update was interrupted, resume |
| **Boot app** | Validating | (boot) | validate_app passes, trials > 0 | step down trials byte | Trial boot after verify, testing new firmware |

### Persistence

- **Step down**: 1→0 bit clear on a single OB byte. Cheap, no erase needed.
- **Refresh**: Full OB erase + rewrite. Required when setting bits from 0→1 or writing new metadata (checksum).
- **None**: State doesn't change, no OB write needed.

### Metadata (stored in option bytes)

| Field | OB Offset | Description |
|-------|-----------|-------------|
| State | +0 | Boot lifecycle state (0xFF/0x7F/0x3F) |
| Trials | +2 | Trial boot counter, each boot clears one bit |
| Checksum | +4,+6 | CRC16 of application firmware |
