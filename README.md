# rust-raytracer

Basic ray-tracer in Rust.


Features done:

- Spheres
- Planes
- Reflection
- Refraction
- Shadows
- Fresnel material
- Multithreading
- Adaptive ray depth
- Triangle intersection
- Obj loading (no materials)

Todo:

- fix random sampler
- static dispatch for materials
- BxDF
- Scenes
- Textures
- Vertex normal interpolation


## Example

Fresnel reflections:
![Test image](examples/fresnel.png)

Obj model loading and triangle intersection:
![Test image](examples/cow.png)

## Build and run

```
cargo run

cargo run --release
```
