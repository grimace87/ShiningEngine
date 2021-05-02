
### Structure overview

There are 3 core components. Everything else is a sub-component of one of these:
- App (the application with its assets and logic)
- Platform (everything that's OS-specific)
- Engine (collection of sub-components; essentially serves to run the App on the Platform)

### Building and Running

Compiling shaders (copied from the Erupt examples, not necessarily optimal):

`glslc -g -O triangle.vert -o triangle.vert.spv`
`glslc -g -O triangle.frag -o triangle.frag.spv`

Running app for Android:

`cargo apk run`

Running on desktop:

`cargo run`

Note that the workspace "default-members" key tells Cargo which binary to run.


### Coordinate systems

Blender likes to think of +Z being the up direction. Vulkan considers -Y
to be the up direction.

To keep things working nicely, models should be created assuming that
+X points to the right and +Y pointing into the screen. When exporting,
the Vulkan system should be specified in the export dialog, that is
-Y is the up direction and +Z is the forward direction.

With regards to texture coordinates, the first texels in the texture's memory
are considered the first row of the image. When loading a JPEG, this will be
the top row, hence coordinate (0,0) is the top-left corner of the image.
Blender however, considers the bottom-left to have coordinate (0,0). Image
data loaded from a JPEG loader should be Y-inverted before loading into
texture memory so that the texture coordinates exported by Blender will match
the image stored in texture memory.
