# Usage (CLI)

The `openlogi` command-line tool. For install and configuration, see the
[README](../README.md).

```sh
openlogi list                    # paired devices: slot, codename, kind, online, battery
openlogi assets sync             # pre-fetch device renders from the fastest available mirror
openlogi backlight               # show MX Keys-style backlight state (HID++ 0x1982)
openlogi backlight off | on      # disable / re-enable it persistently
openlogi snapshot [PATH]         # capture one webcam frame to PNG (default: snapshot.png)
openlogi camera get              # each UVC control's min/max/default/current
openlogi light list              # recognized standalone lights and their controls
openlogi diag features           # dump every HID++ feature the active device reports
```

Running `openlogi` with no subcommand defaults to `list`. Set
`OPENLOGI_LOG=debug` for verbose tracing in the CLI, GUI, or agent.

`list` exits with status `2` when the scan succeeded but found no Logitech
HID++ device and no webcam — a plain "nothing attached", not a syntax error.
Every other subcommand either succeeds (`0`) or fails outright.

## Keyboard backlight

`backlight` drives the white, level-adjustable backlight on the MX Keys line.
RGB keyboards use `0x8070` / `0x8080` instead and are driven by
`diag lighting`. Add `--device NAME` (a case-insensitive substring of the
device name) when several keyboards are paired.

## Camera

```sh
openlogi camera get                          # read all controls (the default action)
openlogi camera set exposure 100             # write one control; persists on the device
openlogi camera set focus_auto 1             # auto-mode toggles take 0/1
openlogi snapshot shot.png --camera <ID>     # capture from a specific camera
```

Controls: `zoom`, `focus`, `exposure`, `power_line_frequency`,
`low_light_compensation`, `brightness`, `contrast`, `saturation`,
`sharpness`, `white_balance`, `tint`, and the auto toggles `focus_auto`,
`exposure_auto`, `white_balance_auto`. `--camera <ID>` selects the camera by
unique id; without it the first Logitech camera is used.

## Lights

Standalone lights (Litra) are controlled outside the HID++ paired-device
model:

```sh
openlogi light list
openlogi light on  --device litra            # --device is a name/identity substring
openlogi light off --device litra
openlogi light brightness --percent 60       # normalized; or --lumens N (native)
openlogi light temperature --kelvin 4000
```

## Diagnostics

`diag` runs real-device round-trip smoke tests against the HID++ write path:
read → write → read back → restore → report. Add `--device NAME`
(case-insensitive substring) to pick the target when several online devices
are paired.

```sh
openlogi diag features           # every HID++ feature the active device reports
openlogi diag controls           # 0x1b04 reprogrammable controls and capability flags
openlogi diag battery            # raw battery report (0x1004 or 0x1000 fields)
openlogi diag dpi                # read → write → read-back → restore DPI (smoke test)
openlogi diag smartshift         # toggle SmartShift and restore (smoke test)
openlogi diag lighting ff0000    # solid colour for a wired RGB keyboard (any RRGGBB hex)
openlogi diag wheel              # read the 0x2121 wheel reporting resolution
```

`diag lighting` accepts `--method auto|effects|perkey|perkeyv2` (default
`auto`: prefer 0x8070 ColorLedEffects, falling back to 0x8081 then 0x8080).
`diag wheel --resolution low|high` writes the resolution directly to the
device; it does not update `config.toml` — use the GUI or TOML for persistent
reconnect behavior.

## Asset synchronization

Asset synchronization probes `assets.openlogi.org`, the versioned Cloudflare
Pages release alias, and the pinned jsDelivr npm release concurrently. The
first mirror with a valid catalog supplies every file for that synchronization
run. Set `OPENLOGI_ASSETS` or pass `openlogi assets sync --base <URL>` to use
one uniform asset origin instead of automatic mirror selection. `--out <DIR>`
overrides the destination directory.
