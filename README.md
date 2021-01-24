# miniaturo

A RAW image thumbnailer written in Rust.

This project is intended to be a drop-in replacement for the (mostly
unmaintained) [raw-thumbnailer] project.

[raw-thumbnailer]: https://libopenraw.freedesktop.org/raw-thumbnailer/

## Implementation notes

miniaturo uses [libopenraw] to parse the RAW image and thus supports all camera
formats that libopenraw supports.

The loading, resizing and encoding of the thumbnail is done in pure Rust using
[image-rs].

[libopenraw]: https://libopenraw.freedesktop.org/
[image-rs]: https://github.com/image-rs/image

## Project name

The name of this project is the Esperanto word for "thumbnail".

## Tests

To run integration tests, first download the test images:

    python3 tests/download-test-images.py

Then run tests:

    cargo test

## License

This project is licensed under the GPLv3 or later.

    Copyright (C) 2021 Danilo Bargen
    
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
