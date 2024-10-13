mq_js_bundle.js contains most common macroquad's dependencies JS code in one JS file.

`mq_js_bundle.js` was made by:

```bash
#!/bin/bash

function wrap_js {
    echo "(function () {" >> mq_js_bundle.js
    cat $1 >> mq_js_bundle.js
    echo "}());" >> mq_js_bundle.js
}
cat ../../miniquad/js/gl.js > mq_js_bundle.js
wrap_js ../../quad-snd/js/audio.js
wrap_js ../../sapp-jsutils/js/sapp_jsutils.js
wrap_js ../../quad-net/js/quad-net.js 
minify mq_js_bundle.js > mq_js_bundle2.js
```
