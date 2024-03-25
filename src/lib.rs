use wasm_bindgen::prelude::*;
use js_sys::{Array, Uint8Array};

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

    #[wasm_bindgen(method, js_name = createExtension)]
    fn create_extension(this: &Document, ctor: &JsValue) -> JsValue;
}

#[wasm_bindgen(module = "@gltf-transform/extensions")]
extern "C" {
    #[wasm_bindgen(js_name = "ALL_EXTENSIONS")]
    static ALL_EXTENSIONS: Array;

    type KHRXMP;

    #[wasm_bindgen(constructor)]
    fn new(document: &Document) -> KHRXMP;

    type Packet;

    #[wasm_bindgen(method, js_name = createPacket)]
    fn create_packet(this: &KHRXMP) -> Packet;

    #[wasm_bindgen(method)]
    fn setContext(this: &Packet, context: &JsValue) -> Packet;

    #[wasm_bindgen(method)]
    fn setProperty(this: &Packet, name: &str, value: &JsValue) -> Packet;
}

#[wasm_bindgen(module = "@gltf-transform/core")]
extern "C" {
    type Root;

    #[wasm_bindgen(method, js_name = getRoot)]
    fn get_root(this: &Document) -> Root;

    #[wasm_bindgen(method, js_name = createExtension)]
    fn create_extension(this: &Root, extension: KHRXMP) -> KHRXMP;

    #[wasm_bindgen(method, js_name = setExtension)]
    fn set_extension(this: &Root, name: &str, extension: &JsValue);
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
pub async fn add_xmp_metadata(input: Uint8Array, author: &str) -> Result<JsValue, JsValue> {
    console_log("Starting XMP metadata addition...");

    let mut io = NodeIO::new();
    io = io.registerExtensions(ALL_EXTENSIONS.clone());

    console_log("Reading binary...");
    let document: Document = io
        .read_binary(&input)
        .await
        .map_err(|err| {
            console_log(&format!("Error reading binary: {:?}", err));
            JsValue::from_str("Failed to read binary")
        })?
        .dyn_into()
        .map_err(|err| {
            console_log(&format!("Error converting to Document: {:?}", err));
            JsValue::from_str("Failed to convert to Document")
        })?;

    console_log("Document read from binary.");

    // Create an Extension attached to the Document.
    console_log("Creating XMP extension...");
    let xmp_extension = KHRXMP::new(&document);
    console_log("XMP extension created.");

    // Create Packet property.
    console_log("Creating XMP packet...");
    let packet = xmp_extension.create_packet();
    console_log("XMP packet created.");


    let context = js_sys::Object::new();
    js_sys::Reflect::set(&context, &"dc".into(), &"http://purl.org/dc/elements/1.1/".into()).unwrap();
    packet.setContext(&context);
    console_log("Context set on XMP packet.");

    let property_value = js_sys::Object::new();
    let list = js_sys::Array::new();
    list.push(&author.into());
    js_sys::Reflect::set(&property_value, &"@list".into(), &list).unwrap();
    packet.setProperty("dc:Creator", &property_value);
    console_log("Property set on XMP packet.");

    // Assign to Document Root.
    document.get_root().set_extension("KHR_xmp_json_ld", &packet.into());
    console_log("XMP extension assigned to document root.");

    console_log("Writing document to binary...");
    let output: JsValue = io.writeBinary(&document).await?;
    console_log("Document written to binary.");

    console_log("XMP metadata addition completed.");

    Ok(output)
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

    document.transform(&js_prune()).await?;
    document.transform(&js_dedup()).await?;

    let output: JsValue = io.writeBinary(&document).await?;

    Ok(output)
}