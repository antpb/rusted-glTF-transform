use wasm_bindgen::prelude::*;
use js_sys::{Array, Uint8Array, Object};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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
}

#[wasm_bindgen(module = "@gltf-transform/functions")]
extern "C" {
    #[wasm_bindgen(js_name = "prune")]
    fn js_prune() -> JsValue;

    #[wasm_bindgen(js_name = "dedup")]
    fn js_dedup() -> JsValue;

    #[wasm_bindgen(js_name = "textureCompress")]
    fn js_texture_compress(options: &JsValue) -> JsValue;
}

#[wasm_bindgen]
pub async fn optimize_textures(input: Uint8Array) -> Result<JsValue, JsValue> {
    console_log("Starting texture optimization...");

    let mut io = NodeIO::new();
    io = io.registerExtensions(ALL_EXTENSIONS.clone());

    let document: Document = io.read_binary(&input).await?.dyn_into()?;

    let options = js_sys::Object::new();

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
	
    console_log("Applying textureCompress transformation...");
    document.transform(&js_texture_compress(&options)).await?;

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

    let dependencies = JsValue::from(js_sys::Object::new());
    io = io.register_dependencies(&dependencies);

    let document: Document = io.read_binary(&input).await?.dyn_into()?;

    document.transform(&js_prune());
    document.transform(&js_dedup());

    let output: JsValue = io.writeBinary(&document).await?;

    Ok(output)
}