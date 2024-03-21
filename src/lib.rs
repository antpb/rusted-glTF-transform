use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::{Array, Uint8Array, Object};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// #[wasm_bindgen]
// pub struct VRMExtension {
//     extension_name: String,
// }

// #[wasm_bindgen]
// impl VRMExtension {
//     #[wasm_bindgen(constructor)]
//     pub fn new(extension_name: String) -> VRMExtension {
//         VRMExtension { extension_name }
//     }

//     #[wasm_bindgen(method)]
//     pub fn register(&self) {
//         // Implement the register method if needed
//         console_log(&format!("Registering extension: {}", self.extension_name));
//     }
// }

#[wasm_bindgen(module = "@gltf-transform/core")]
extern "C" {
    type Document;
    type NodeIO;

    #[wasm_bindgen(constructor)]
    fn new() -> NodeIO;

    #[wasm_bindgen(method)]
    fn registerExtensions(this: &NodeIO, extensions: Array) -> NodeIO;

    #[wasm_bindgen(catch, method, js_name = readBinary)]
    async fn read_binary(this: &NodeIO, data: &Uint8Array) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, method)]
    async fn writeBinary(this: &NodeIO, document: &Document) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, method)]
    async fn transform(this: &Document, transform: &JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(method, js_name = registerDependencies)]
    fn register_dependencies(this: &NodeIO, dependencies: &JsValue) -> NodeIO;
}

#[wasm_bindgen(module = "@gltf-transform/extensions")]
extern "C" {
    #[wasm_bindgen(js_name = "ALL_EXTENSIONS")]
    static ALL_EXTENSIONS: Array;

    #[wasm_bindgen(js_name = "MeshoptEncoder")]
    static MESHOPT_ENCODER: JsValue;
}

//@todo WIP need to figure out vrm extensions translating over.
// #[wasm_bindgen]
// pub fn create_vrm_extensions() -> Array {
//     let vrm_extension_names = &[
//         "VRMC_vrm",
//         "VRMC_vrm_animation",
//         "VRMC_node_constraint",
//         "VRMC_springBone",
//         "VRMC_materials_mtoon",
//     ];

//     let vrm_extensions = Array::new();

//     for name in vrm_extension_names {
//         let extension = VRMExtension::new(name.to_string());
//         let js_extension = JsValue::from(extension);
//         vrm_extensions.push(&js_extension);
//     }

//     console_log(&format!("Created VRM extensions: {:?}", vrm_extensions));

//     vrm_extensions
// }

#[wasm_bindgen(module = "sharp")]
extern "C" {
    type Sharp;

    #[wasm_bindgen(js_name = "default")]
    fn sharp() -> Sharp;
}

#[wasm_bindgen(module = "@gltf-transform/functions")]
extern "C" {
    #[wasm_bindgen(js_name = "prune")]
    fn js_prune() -> JsValue;

    #[wasm_bindgen(js_name = "dedup")]
    fn js_dedup() -> JsValue;

    #[wasm_bindgen(js_name = "reorder")]
    fn js_reorder() -> JsValue;

	#[wasm_bindgen(js_name = "textureCompress")]
    fn js_texture_compress(options: &JsValue) -> JsValue;

}

#[wasm_bindgen]
pub async fn optimize_textures(input: Uint8Array) -> Result<JsValue, JsValue> {
    console_log("Starting texture optimization...");

    let mut io = NodeIO::new();
    io = io.registerExtensions(ALL_EXTENSIONS.clone());
    io = io.registerExtensions(create_vrm_extensions());

    // Read the document from the input Uint8Array
    let document: Document = io.read_binary(&input).await?.dyn_into()?;

    // Create the options object for textureCompress
    let options = js_sys::Object::new();

	// @todo fix this with a separate sharp module likely.
    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     console_log("Running in Node.js environment, using sharp encoder...");
    //     let sharp_module = js_sys::dynamic_import("sharp").await?;
    //     let sharp_encoder = js_sys::Reflect::get(&sharp_module, &JsValue::from("default"))?;
    //     js_sys::Reflect::set(&options, &JsValue::from("encoder"), &sharp_encoder)?;
    //     console_log("Sharp encoder set.");
    // }

    js_sys::Reflect::set(&options, &JsValue::from("targetFormat"), &JsValue::from("webp"))?;
    console_log("Target format set to WebP.");

    let quality = JsValue::from(70);
    js_sys::Reflect::set(&options, &JsValue::from("quality"), &quality)?;
    console_log("Quality set to 70.");

    let resize = js_sys::Array::new();
    resize.push(&JsValue::from(512));
    resize.push(&JsValue::from(512));
    js_sys::Reflect::set(&options, &JsValue::from("resize"), &resize)?;
    console_log("Resize set to [512, 512].");

	// @todo make this more configurable. Default all for now.
    // let slots_regex = js_sys::RegExp::new("/^(?!normalTexture).*$/", "i");
    // js_sys::Reflect::set(&options, &JsValue::from("slots"), &slots_regex)?;
    // console_log("Excluding normal textures from compression.");
	
	// log the final options
    console_log("Applying textureCompress transformation...");
	// document.transform(&js_texture_compress(&options));
	document.transform(&js_texture_compress(&options)).await?;
	// @todo look into prune an dedup later currently working but should be around some config options.
	// document.transform(&js_prune());
    // document.transform(&js_dedup());

    // Write the optimized document to binary
    console_log("Writing optimized document to binary...");
    let output: JsValue = io.writeBinary(&document).await?;

    console_log("Texture optimization completed.");

    Ok(output)
}

fn console_log(message: &str) {
    log(message);
}

#[wasm_bindgen]
pub async fn optimize_gltf(input: Uint8Array) -> Result<JsValue, JsValue> {
    let mut io = NodeIO::new();
    io = io.registerExtensions(ALL_EXTENSIONS.clone());

    // Register MeshoptEncoder as a dependency
    let dependencies = JsValue::from(js_sys::Object::new());
    js_sys::Reflect::set(&dependencies, &JsValue::from("meshoptimizer"), &MESHOPT_ENCODER)?;
    io = io.register_dependencies(&dependencies);

    // Read the document from the input Uint8Array
    let document: Document = io.read_binary(&input).await?.dyn_into()?;

    // Apply transformations to the document
    document.transform(&js_prune());
    document.transform(&js_dedup());
    // document.transform(&js_reorder());

    // Write the optimized document to binary
    let output: JsValue = io.writeBinary(&document).await?;

    Ok(output)
}