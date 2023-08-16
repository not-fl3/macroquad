# Reimagining macroquad

*Disclaimer on this branch*

The idea here - anything goes, the more ideas are going to be tried - the better. Implement first, think later kind of thing.

Code here does not work, not intended to work and will never work!

This is just a playground to experiment with the ideas outlined in this file.

# The goal

- eleminate UB
- API for fine-tuned batching, allowing non-batched or instanced draw calls
- keep all 0.3 API under macroquad::compatibility or macroquad::immediate_mode

# Details

## Rendering

In 0.3 macroquad all draw calls are always, automatically batched.

draw_whatever(); will always do a `quad_gl` call, appending some vertices to some already existing gpu buffer to be rendered later. This takes into account materials, textures etc, making new buffers when needed.

So this is basically a dynamic batching for all the meshes all the time.
Which is fine when most meshes are 2D sprites with the same textures, but
it does not work for 3d or more complicated 2d scenes.

The main goal: allow dynamic, static and no batching for draw calls.

So far the idea is:

- destroy all draw_* functions and use a builder pattern.

Like this:
```rust
// this will create a nice CPU-only sprite
let sprite = Square::new()
  .pos(vec2(x, y))
  .texture(texture);

// this will upload a model to GPU and will make it drawable by itself
let model = sprite.stateful_gpu_object();

// will draw right away, without any batching
scene_graph().draw(&model);

// acquire an old good `guad_gl` instance, basically
let sprite_layer = scene_graph().sprite_layer();

// do what draw_texture() did in 0.3
sprite_layer.append(sprite);
sprite_layer.append(sprite.pos(vec2(x + 10, y)));
sprite_layer.append(some_text);

// do what macroquad did internally when batching got breaked by a new material
// or on the end of the frame
scene_graph().draw(sprite_layer);
```

## Compatibility with 0.3

To keep compatibility, something like `macroquad::im_legacy_old_api` may be introduced.
It will just wrap new builder calls into old draw_* calls on one single sprite_layer.

## Design thoughts

macroquad's main loop, scenes and coroutines were nice, but, maybe, its time to let them go.

No one really use scenes/coroutines anyway.

So it may be a good idea to keep macroquad pure, just rendering, assets, sounds etc,
but do not touch game's architecture.

Instead of `scene` as a thing for game nodes, macroquad will have its own "scene_graph" (scene_graph() thing from previous section). And games will take some sort of handles from macroquad into their own "ecs","nodes" or whatever.

## "Plugin" crates

macroquad 0.4 will eleminate default features.

in macroquad-0.3 adding a dependency depending on macroquad silently enable all macroquad's default features:

```toml
macroquad = {version = "0.3", default-features = false, features = ["3d", "audio"]}
macroquad_tiled = "0.3"
```
cargo resolves this into basically `default-features = true`, very un-intuitive. There are some workarounds, but better just give up default features.

Also "macroquad-*" should find new home, having everything in the macroquad repo was a bad idea.
