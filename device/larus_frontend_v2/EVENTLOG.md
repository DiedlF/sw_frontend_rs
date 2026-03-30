# EVENTLOG.CSV

`EVENTLOG.CSV` is an append-only SD-card event log for inflight diagnostics.

## Format

```text
timestamp_us,event,a,b,c,d
```

- `timestamp_us` — frontend uptime timestamp in microseconds
- `event` — event name
- `a..d` — event-specific integer payload fields

All payload fields are signed integers.

## Event definitions

### thermal_reset
Thermal center estimator was reset.

- `a` — reset reason
  - `1` = not circling
  - `2` = no GPS
  - `3` = GPS ground speed too low
- `b` — sample count before reset
- `c` — accumulated turn angle in degrees
- `d` — reserved

### thermal_circle_complete
A full circling segment was completed and the estimator produced a result.

- `a` — sample count
- `b` — estimated center distance in decimeters
- `c` — confidence in permille (`0..1000`)
- `d` — reserved

### thermal_est_valid
Thermal center estimate changed from invalid to valid.

- `a` — signed east shift in decimeters
- `b` — signed north shift in decimeters
- `c` — distance in decimeters
- `d` — confidence in permille

### thermal_est_invalid
Thermal center estimate changed from valid to invalid.

- `a` — signed east shift in decimeters
- `b` — signed north shift in decimeters
- `c` — distance in decimeters
- `d` — confidence in permille

### fly_mode_changed
Frontend changed fly mode.

- `a` — previous `FlyMode`
- `b` — new `FlyMode`
- `c` — absolute turn rate in millirad/s
- `d` — GPS ground speed in decimeters/s

Current `FlyMode` values:
- `0` = StraightFlight
- `1` = Circling

### gps_state_changed
GPS state changed.

- `a` — previous `GpsState`
- `b` — new `GpsState`
- `c` — satellite count
- `d` — reserved

Current `GpsState` values:
- `0` = NoGps
- `1` = PosAvail
- `2` = HeadingAvail

### update_found
A valid firmware update image was found on the SD card.

- `a` — version major
- `b` — version minor
- `c` — version patch
- `d` — version build

### update_install_start
Firmware installation started.

- `a` — new app address
- `b` — new app length
- `c` — destination address
- `d` — copy function address

## Notes

- The log is intentionally event-based, not sample-based.
- It is meant for post-flight diagnostics without a debugger.
- Use it together with `PANIC.LOG` for reset/crash analysis.
