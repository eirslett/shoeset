const js = import(`./node_modules/@eirslett/sjuz/sjuz.js`);

fetch('/vatsim.7z').then(async response => {
    const data = await response.arrayBuffer();
    const mod = await js;
    const result = mod.decompress(new Uint8Array(data));
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
