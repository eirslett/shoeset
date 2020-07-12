import * as shoeset from '@eirslett/shoeset';
import fs from 'fs';

const archive = fs.readFileSync('foobar.7z');
const decompressed = shoeset.default.decompress(archive);
for (const file of decompressed.files) {
    console.log('name', file.name);
    console.log('data', new TextDecoder().decode(file.data));
}
