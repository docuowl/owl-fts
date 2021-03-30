# owl-fts

`owl-fts` is a full-text search parser used by Docuowl.

Docuowl allows FTS by appending an encoded binary representation of the FTS 
index to the page's header, allowing scripts to read and interpret it. This 
repository contains Rust sources responsible for parsing those contents.

## Protocol Documentation
The first data level is found as a Base64-encoded binary representation of the
initial contents. It is composed by a set of 4 magic bytes, followed by a 32-bit
big-endian unsigned integer representing the length of a subsequent Brotli 
stream, which by its turn contains all remaining index's data.

```
|-    Magic    -|- Brot Begin Indicator -|- Brot Length -|- Brot Stream -|
|  6F 77 6C 00  |            01          |  ?? ?? ?? ??  |       ...     |
\______________/^\______________________/^\_____________/^\______________/
```

1. `Magic` will **always** contain the sequence `0x6F 0x77 0x6C 0x00`.
2. `Brot Begin Indicator`, is a single `0x01` byte indicating the begin of the 
Brotli stream.
3. `Brot Length` contains a 32-bit big-endian unsigned integer representing the
amount of bytes that compose the following Brotli Stream.
4. `Brot Stream` contains a deflated binary value of size `Brot Length`.

After inflating the stream, the following structure will be present:

```
|- Section Indexes Indicator -|- Section Name -| Separator -|
|              02             |       ...      |     00     |
\____________________________/^\______________/^\__________/

```

1. `Section Indexes Indicator` will always contain an `0x02` value indicating
the begin of this section.
2. `Section Name` is an UTF-8 string of arbitrary size terminated by a `NULL`
byte.

After each `NULL` byte, the client must check whether the subsequent byte is 
`0x03`, indicating that the next section will begin. Otherwise, another section
name will follow.

```
|- Index Separator -|- Word Length -|- Cluster Length -|- Word -|- Page Size -|- Page Array -|
|         03        |       ??      |       ??         |   ...  |      ??     |      ...     |
\__________________/^\_____________/^\________________/^\______/^\___________/^\_____________/
```

1. `Index Separator` will always contain an `0x03` value indicating
the begin of this section.
2. `Word Length` contains an 8-bit unsigned integer indicating the length of
each word in this section.
3. `Word` contains an UTF-8 string of `Word Length` bytes, with NO `NULL` 
terminator.
4. Page Size contains an 8-bit unsigned integer indicating how many items will
be present in `Page Array`.
5. `Page Array` contains tuples of information as depicted below.

After `Word Length` is consumed, the client must determine whether the stream 
contains more bytes, indicating that another section of this same kind (with no
`0x03` separator) follows.

```
|- Page Index -|- Frequency -|
|     00 00    |    00 00    |
\_____________/^\____________/
```

1. Page Index is a 16-bit big-endian unsigned integer representing an index of a 
page contained within the section depicted by the first graph on this document.
2. Frequency is a 16-bit big-endian unsigned integer indicating how many times
this given word appears in `Page Index`.

## License

```
MIT License

Copyright © 2021 Victor Gama
Copyright © 2021 Real Artists

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
