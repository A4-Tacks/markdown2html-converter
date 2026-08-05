// Rewrite each `src:` list of KaTeX's stylesheet into a single woff2 `data` URL, dropping the woff and ttf fallbacks.
//
// Usage: node tools/build-katex-css.js <katex-dist-directory> > resources/katex.min.css
//
// The directory is the `dist` of the KaTeX package, so it holds `katex.min.css` and a `fonts` directory.

const fs = require('fs');
const path = require('path');

const dist = process.argv[2];

if (!dist) {
    throw new Error('usage: node tools/build-katex-css.js <katex-dist-directory>');
}

const css = fs.readFileSync(path.join(dist, 'katex.min.css'), 'utf8');

let rewritten = 0;

// Matches the whole `src:` declaration of a @font-face block, whose first entry is always the woff2 file.
const out = css.replace(
    /src:url\(fonts\/([A-Za-z_0-9-]+\.woff2)\) format\("woff2"\)(?:,url\(fonts\/[A-Za-z_0-9-]+\.[a-z0-9]+\) format\("[^"]+"\))*/g,
    (_match, file) => {
        const data = fs.readFileSync(path.join(dist, 'fonts', file)).toString('base64');

        rewritten += 1;

        return `src:url(data:font/woff2;base64,${data}) format("woff2")`;
    },
);

if (rewritten === 0) {
    throw new Error('no @font-face block was rewritten');
}

if (/url\(fonts\//.test(out)) {
    throw new Error('a relative font URL survived the rewrite');
}

process.stderr.write(`rewrote ${rewritten} @font-face blocks\n`);
process.stdout.write(out);
