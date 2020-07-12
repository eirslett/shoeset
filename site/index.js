const js = import(`./node_modules/@eirslett/shoeset/shoeset.js`);

fetch('/foobar.7z').then(async response => {
    const data = await response.arrayBuffer();
    const mod = await js;

    const before = performance.now();
    const result = mod.decompress(new Uint8Array(data));
    const after = performance.now();
    console.log('duration', after - before);

    for (const file of result.files) {
        console.log('name', file.name);
        console.log('data', file.data);

        console.log(new TextDecoder().decode(file.data));
    }
});
