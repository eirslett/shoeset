const js = import(`./node_modules/@eirslett/sjuz/sjuz.js`);

fetch('/foobar.7z').then(async response => {
    const data = await response.arrayBuffer();
    const mod = await js;
    const result = mod.decompress(new Uint8Array(data));
    console.log('got result', result);
});

console.log('hello world');
