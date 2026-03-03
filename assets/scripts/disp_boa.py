#!/usr/bin/env python3
"""
Display BOA Information Script

This script sends the 'disp boa {slotid}' command to query BOA (Board On-line Aging)
information for a specific slot on Huawei devices.

@params
- slotid: string
  description: Slot ID (e.g., 1, 2, 1/0)
  required: true
@end_params
"""

from bspterm import current_terminal, params


def main():
    term = current_terminal()
    slotid = params.slotid

    term.send(f"disp boa {slotid}\n")


if __name__ == "__main__":
    main()
