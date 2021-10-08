PNG2BIN
=======

Converts PNG images to firmware friendly BIN format. As this is a single use tool, it is written in a quick'n'dirty way.

Usage
=====

`png2bin <dir>` - will conert each _png_ file in _dir_ into bin file

Format description
==================

* First two bytes - length of the encoded data in bytes, little endian
* data - compressed 1BPP image.

Image is scanned from left to right, top to bottom. Each pixel
is represented as a bit in a data stream and will be set to 0 for
black pixel and 1 for any other color. In case amount of pixels is not
dividable by 8, missing pixels will be stuffed with value 1. Resulting
bitstream will be compressed using LSZZ algorithm.

The image dimensions are not stored and firmware is supposed to know
anticipated image dimensions.