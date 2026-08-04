window.MathJax = {
    tex: {
        inlineMath: [['\\(', '\\)']],
        displayMath: [['\\[', '\\]']]
    },
    options: {
        // Nothing is typeset unless it carries the `math` class, so plain text containing `$` is never touched.
        ignoreHtmlClass: '.*',
        processHtmlClass: 'math',
        // Enrichment starts a worker which downloads the speech rule engine, and that would break an offline file.
        menuOptions: {
            settings: {
                enrich: false,
                speech: false,
                braille: false
            }
        }
    },
    startup: {
        pageReady: function () {
            document.querySelectorAll('[data-math-style]').forEach(function (element) {
                var display = element.getAttribute('data-math-style') === 'display';
                var math = document.createElement('span');

                math.className = 'math';
                math.textContent = display ? '\\[' + element.textContent + '\\]' : '\\(' + element.textContent + '\\)';

                // A ```math block becomes `<pre><code>`, and the whole `<pre>` has to be replaced.
                var target = element.parentNode.tagName === 'PRE' ? element.parentNode : element;

                target.parentNode.replaceChild(math, target);
            });

            return MathJax.startup.defaultPageReady();
        }
    }
};
