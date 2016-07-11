# XRay
Simple render

# Done
* PT (two sampling strategies)
* Lambert/Phong (for both sampling strategies)
* PT with MIS
* DF
* CPU multithreading

# In progress
* BDPT with full path join on CPU

# Roadmap
* BSDF (not only specular glass!)
* BDPT with MIS on CPU
* PT on GPU
* PT with MIS on GPU
* BDPT on GPU
* BDPT with MIS on GPU
* Interactivity
* Env map lighting
* Load scene from file
* BVH (don't know how to use with DF)
* SBDPT on CPU
* SBDPT with MIS on CPU
* SBDPT on GPU
* SBDPT with MIS on GPU
* BSSRDF
* Tonemapping
* Don't leave Rust

# How to build
1. Download latest stable Rust.
2. Install SFML and CSFML.
3. Run `cargo build --release`
