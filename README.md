*WARNING:* This library is highly experimental, and may introduce breaking changes.

# shoeset

This is a 7z archive decompressor, written in Rust.
The code is designed to be portable:

- Run natively as a binary
- Run as WebAssembly in a web browser or Node.js

Supported decompression algorithms:
- LZMA
- LZMA2

Features of 7z that are *not* supported:
- Archive compression
- CRC validation
- Other decompression algorithms

The library is called "shoeset" because that's approximately how you pronounce "7z" in Norwegian.

### Usage with JavaScript/WebAssembly

Installation:

``
npm install @eirslett/shoeset
``

From Node.js:

```
import * as shoeset from '@eirslett/shoeset';
import fs from 'fs';

const archive = fs.readFileSync('foobar.7z');
const decompressed = shoeset.default.decompress(archive);
for (const file of decompressed.files) {
    console.log('name', file.name);
    console.log('data', file.data);
    
    // If the file is UTF-8 encoded, we can log it as a string:
    console.log(new TextDecoder().decode(file.data));
}
```

From the browser:
```
const js = import(`./node_modules/@eirslett/shoeset/shoeset.js`);

fetch('/foobar.7z').then(async response => {
    const data = await response.arrayBuffer();
    const mod = await js;

    const result = mod.decompress(new Uint8Array(data));

    for (const file of result.files) {
        console.log('name', file.name);
        console.log('data', file.data);

        // If the file is UTF-8 encoded, we can log it as a string:
        console.log(new TextDecoder().decode(file.data));
    }
});
```

## Building

1) Setup rust on your development machine, for example with [rustup](https://rustup.rs/).
2) git clone this repository
3) Run `cargo build` to build the native binary, or `./build-npm.sh` to build the npm package
4) Run `cargo test` to run the unit tests, or `./test.sh` to test the npm package

#### Publishing to NPM

1) `./build-npm.sh`
2) `cd pkg`
3) `npm publish --access public`

## Local development setup

1) Run `./build-npm.sh`
2) `cd pkg`
3) `npm link`
4) `cd ../site`
5) `npm link @eirslett/shoeset`
6) `node test-nodejs.mjs` to check if the code is working

## Contributing

Just send me a pull request. I cannot guarantee that I have time to review it, if there are many PRs.
