HarambeJTAG firmware
===

State: experimental, UART bridge works

Building
---

```
$ cd dvt1
$ cargo build --release
```

Flashing
---

You'll need elf2uf2-rs (eg. from `cargo install elf2uf2-rs`).

Start the board in bootrom mode (hold down BOOT while resetting the device). Mount the USB drive that appears. Then:

```
$ cd dvt1
$ cargo run --release
```

This will use elf2uf2-rs to convert the program into a UF2 file and copy it onto the mounted storage. Unmount the storage and the device should run the newly flashed software.

Usage
---

Two serial devices will appear: the first one is an unused command/monitoring device. The second is a 115200 baud UART bridge to the iPod's UART pins.

License
---

Copyright (C) 2024 Serge Bazanski

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
