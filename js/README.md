mq_js_bundle.js contains most common macroquad's dependencies JS code in one JS file.

`mq_js_bundle.js` was made by:

```bash
cat ../../miniquad/js/gl.js > mq_js_bundle.js
cat ../../quad-snd/js/audio.js >> mq_js_bundle.js
cat ../../sapp-jsutils/js/sapp_jsutils.js >> mq_js_bundle.js
cat ../../quad-net/js/quad-net.js >> mq_js_bundle.js
minify mq_js_bundle.js
```
