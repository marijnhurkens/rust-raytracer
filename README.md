# rust-raytracer

Basic ray-tracer in Rust, following the [PBRT Book](https://pbr-book.org/3ed-2018/contents).

Features done:

- Objects (planes, triangles)
- Meshes
- Obj loading
- Lights (point, area, distant)
- Materials (matte, half of plastic)
- Multithreading
- Multiple Importance Sampling
- Denoising using OpenImage Denoise
- Basic scene configuration

Todo:

- Microfacets
- Transmittance (BTDF)
- File output
- Textures
- Other things

## Build and run

You need a copy of Intel® Open Image Denoise (IOID). Grab a package from their 
[download section](https://www.openimagedenoise.org/downloads.html). Unpack this 
somewhere. We refer to this below as the OIDN location.

Export the OIDN location for the build to find the headers & libraries. For example:

```
export OIDN_DIR=$HOME/Downloads/oidn-1.3.0.x86_64.linux/
```

```
cargo run --release ./scene/buddha
```


## Usage

```
rust-raytracer 

USAGE:
    rust-raytracer [ARGS]

ARGS:
    <SCENE_FILE>       
    <SETTINGS_FILE>    

OPTIONS:
    -h, --help    Print help information
```

During rendering, press and hold D for debug layers (probably nothing will show)
and N for normals.

## Examples

Part of PBRT book implementation:

![Buddha](examples/buddha.png)

Old whitted raytracer:

Fresnel reflections:
![Test image](examples/fresnel.png)

Obj model loading and triangle intersection:
![Test image](examples/cow.png)