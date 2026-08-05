document.querySelectorAll('[data-math-style]').forEach(function (element) {
    var display = element.getAttribute('data-math-style') === 'display';
    var math = document.createElement('span');

    // A ```math block becomes `<pre><code>`, and the whole `<pre>` has to be replaced.
    var target = element.parentNode.tagName === 'PRE' ? element.parentNode : element;

    // Without this a single broken formula would throw and leave the rest of the document unrendered.
    katex.render(element.textContent, math, { displayMode: display, throwOnError: false });

    target.parentNode.replaceChild(math, target);
});
