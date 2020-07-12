const js = import(`./node_modules/@eirslett/shoeset/shoeset.js`);

fetch('/big-test-archive.7z').then(async response => {
    const data = await response.arrayBuffer();
    const mod = await js;

    const before = performance.now();
    const result = mod.decompress(new Uint8Array(data));
    const after = performance.now();
    console.log('duration', after - before);
    console.log('got result', result);
    window.result = result;
    console.log('id', result.id);
    console.log('files', result.files);

    for (const file of result.files) {
        console.log('name', file.name);
        console.log('data', file.data);

        console.log(new TextDecoder('latin1').decode(file.data));
    }
});

console.log('hello world');
