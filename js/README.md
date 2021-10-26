audio.js is macroquad's native JS glue for audio support.

mq_js_bundle.js contains most common macroquad's dependencies JS code in one JS file. 

`mq_js_bundle.js` was made by yourself:

```bash
cat miniquad/native/sapp-wasm/js/gl.js >> mq_js_bundle.js
cat macroquad/js/audio.js >> mq_js_bundle.js
cat sapp-jsutils/js/sapp_jsutils.js >> mq_js_bundle.js
cat quad-net/js/quad-net.js >> mq_js_bundle.js
minify mq_js_bundle.js
```
