# A naïve N-body simulation using [wgpu](https://wgpu.rs/)

This project was written mostly as an excuse to learn more about compute shaders and graphics programming in general - something that I think I've generally succeeded at.

I unfortunately don't have the time to implement more efficient algorithm, like Barnes-Hut or the Fast Multipole Method (especially due to the complexity of implementing them on the GPU).

If you're here because you're trying to write something like this yourself, you might be interested in these papers ([1](https://ieeexplore.ieee.org/document/9206962), [2](https://iss.oden.utexas.edu/Publications/Papers/burtscher11.pdf))

### Project Structure

Most of the code is fairly simple, so I won't delve too deep into it.
Here are some pointers to navigating it, though. The code for the initial placement of the bodies is located at the bottom of [sim.rs](src/sim.rs). This currently uses a fairly simple method I wrote myself, which really isn't backed up by anything.
These bodies are then separated into components, loaded into buffers and sent to the [compute shader](src/comp_shader.wgsl). There the actual simulation is performed, which then passes a buffer with all the positions to the [renderer](src/renderer.rs).
This code has two stages (passes). The first simply draws the bodies, as circles with some amount of glow and a 1/255 alpha. The blending is purely additive, which allows me to use the alpha as an indicator of how many bodies are located at that specific pixel.
This allows the second pass to recolor every pixel according to the magma color map. The output is then copied to a buffer and saved to ./output, as a jpeg file.

# Showcase

