"""Renode capture sequence for Moire Generator (not yet run).

Run from xous-dev-toolkit:
    python3 scripts/renode_capture.py --app moire --script ../precursor-moire/capture.py

Contract: run(ctl, outdir) -> number of failed steps. The app is already launched.
"""
import os


def run(ctl, outdir) -> int:
    failures = 0

    def shot(name):
        nonlocal failures
        if not ctl.try_screenshot(os.path.join(outdir, name)):
            failures += 1

    shot("01_menu.png")
    for _ in range(10):                 # down to [ Render ]
        ctl.timed_key("Down", after=0.3)
    ctl.inject_line("")
    ctl.sleep(5.0)
    shot("02_default_lines.png")
    ctl.timed_key("Right", after=5.0)   # nudge layer B by 1 degree
    shot("03_nudged.png")
    ctl.timed_key("BackSpace", after=1.0)
    ctl.timed_key("Down", after=0.5)    # [ Gallery ]
    ctl.inject_line("")
    ctl.sleep(1.0)
    shot("04_gallery.png")
    for _ in range(4):
        ctl.timed_key("Down", after=0.3)
    ctl.inject_line("")                 # Zebra
    ctl.sleep(5.0)
    shot("05_zebra.png")
    return failures
