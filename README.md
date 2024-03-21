### rusted-gltf-transform (EXTREMELY WIP)
rusted-gltf-transform is a Rust/WASM module that provides functionality for optimizing glTF files using the gltf-transform library. It allows you to perform various optimizations on glTF models, including texture optimization and general glTF optimization.


![screenshot](https://3ov.xyz/wp-content/uploads/2024/03/wasm-gltf-transform-preview-scaled.jpg)

## Features
• Texture optimization: Compress textures to WebP format, resize textures, and apply quality settings.
• glTF optimization: Prune unused data, deduplicate data, and apply other optimizations to reduce the size of glTF files.
• WASM-based: The module is compiled to WebAssembly, allowing it to be used in web applications and native apps.

## Building the WASM Module
To build the module, follow these steps:

## Install Rust and wasm-pack:
Install Rust
Install wasm-pack
Clone the repository:

`git clone https://github.com/antpb/rusted-gltf-transform.git`

`cd rusted-gltf-transform`


## Build the WASM module:

```wasm-pack build --target web```
This command will compile the Rust code to WASM and generate the necessary JavaScript bindings. The output will be placed in the pkg directory.


## Using the WASM Module in a JS app
To use the module in your JavaScript app, follow these steps:

Install the module as a dependency in your project:

```npm install path/to/rusted-gltf-transform/pkg```

Import the module in your JS code:

```js
import init, { optimize_textures, optimize_gltf } from 'rusted-gltf-transform';

async function optimizeModel(input) {
  await init();

  // Optimize textures
  const optimizedTextures = await optimize_textures(input);

  // Optimize glTF
  const optimizedGltf = await optimize_gltf(input);

  // Return the optimized glTF
  return optimizedGltf;
}
```

In this example, the optimizeModel function takes the input glTF file as a Uint8Array and returns the optimized glTF as a JavaScript value.
Use the optimized glTF in your application as needed.

## Configuration
The module provides some configuration options for texture optimization:

targetFormat: The target format for texture compression (default: "webp").
quality: The quality setting for texture compression (default: 70).
resize: The dimensions to resize textures to (default: [512, 512]).
You can modify these options by updating the corresponding values in the optimize_textures function in the Rust code.

## License
This project is licensed under the MIT License.

