use wasm_bindgen::prelude::*;
use web_sys::{console, File, FileReader, HtmlCanvasElement, CanvasRenderingContext2d};
use serde::{Deserialize, Serialize};
use rust_core::{ImageOptimizer as CoreImageOptimizer, MeshOptimizer as CoreMeshOptimizer, OptConfig as CoreOptConfig, image::ImageFormat};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Set up better panic messages in debug mode
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"Pixie Juice Web initialized with real processing".into());
}

// Logging utilities
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// WASM-compatible configuration wrapper
#[derive(Serialize, Deserialize, Clone)]
#[wasm_bindgen]
pub struct WasmOptConfig {
    quality: u8,
    preserve_metadata: bool,
    lossless: bool,
    reduce_colors: bool,
    target_reduction: f32,  // Target reduction ratio (0.0-1.0)
}

#[wasm_bindgen]
impl WasmOptConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmOptConfig {
        WasmOptConfig {
            quality: 85,
            preserve_metadata: false,  // Default to removing metadata for better compression
            lossless: false,
            reduce_colors: false,
            target_reduction: 0.0,  // No specific reduction target by default
        }
    }

    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> u8 { self.quality }
    #[wasm_bindgen(setter)]
    pub fn set_quality(&mut self, quality: u8) { self.quality = quality; }

    #[wasm_bindgen(getter)]
    pub fn preserve_metadata(&self) -> bool { self.preserve_metadata }
    #[wasm_bindgen(setter)]
    pub fn set_preserve_metadata(&mut self, preserve: bool) { self.preserve_metadata = preserve; }

    #[wasm_bindgen(getter)]
    pub fn lossless(&self) -> bool { self.lossless }
    #[wasm_bindgen(setter)]
    pub fn set_lossless(&mut self, lossless: bool) { self.lossless = lossless; }

    #[wasm_bindgen(getter)]
    pub fn reduce_colors(&self) -> bool { self.reduce_colors }
    #[wasm_bindgen(setter)]
    pub fn set_reduce_colors(&mut self, reduce_colors: bool) { self.reduce_colors = reduce_colors; }

    #[wasm_bindgen(getter)]
    pub fn target_reduction(&self) -> f32 { self.target_reduction }
    #[wasm_bindgen(setter)]
    pub fn set_target_reduction(&mut self, target_reduction: f32) { 
        self.target_reduction = target_reduction.clamp(0.0, 1.0); 
    }
}

impl From<&WasmOptConfig> for CoreOptConfig {
    fn from(wasm_config: &WasmOptConfig) -> Self {
        CoreOptConfig {
            quality: Some(wasm_config.quality),
            compression_level: None,
            lossless: Some(wasm_config.lossless),
            preserve_metadata: Some(wasm_config.preserve_metadata),
            fast_mode: None,
            reduce_colors: Some(wasm_config.reduce_colors),
            target_reduction: if wasm_config.target_reduction > 0.0 { 
                Some(wasm_config.target_reduction) 
            } else { 
                None 
            },
            max_width: None,
            max_height: None,
        }
    }
}

// Image information structure
#[derive(Serialize, Deserialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: usize,
}

// Main WASM bindings for image processing
#[wasm_bindgen]
pub struct ImageOptimizer {
    core_optimizer: CoreImageOptimizer,
}

#[wasm_bindgen]
impl ImageOptimizer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ImageOptimizer {
        console_log!("Creating new ImageOptimizer with real rust_core backend");
        ImageOptimizer {
            core_optimizer: CoreImageOptimizer::new(),
        }
    }

    /// Process an image file from browser File API using real optimization
    #[wasm_bindgen]
    pub async fn process_image_file(
        &self,
        file_data: &[u8],
        input_format: &str,
        output_format: &str,
        config: &WasmOptConfig,
    ) -> Result<Vec<u8>, JsValue> {
        console_log!("Processing image: {} -> {} ({} bytes)", input_format, output_format, file_data.len());
        
        let core_config = CoreOptConfig::from(config);
        
        // If input and output formats are the same, just optimize
        if input_format == output_format {
            match self.core_optimizer.optimize(file_data, &core_config) {
                Ok(optimized_data) => {
                    console_log!("Image optimized successfully: {} bytes -> {} bytes", 
                        file_data.len(), optimized_data.len());
                    Ok(optimized_data)
                },
                Err(e) => {
                    console_log!("Image optimization failed: {}", e);
                    Err(JsValue::from_str(&format!("Optimization failed: {}", e)))
                }
            }
        } else {
            // Convert to target format and optimize
            let target_format = match output_format {
                "png" => ImageFormat::PNG,
                "jpeg" | "jpg" => ImageFormat::JPEG,
                "webp" => ImageFormat::WebP,
                "gif" => ImageFormat::GIF,
                "bmp" => ImageFormat::BMP,
                "tiff" => ImageFormat::TIFF,
                _ => return Err(JsValue::from_str(&format!("Unsupported output format: {}", output_format)))
            };

            match self.core_optimizer.optimize_to_format(file_data, target_format, &core_config) {
                Ok(converted_data) => {
                    console_log!("Image converted and optimized successfully: {} bytes -> {} bytes", 
                        file_data.len(), converted_data.len());
                    Ok(converted_data)
                },
                Err(e) => {
                    console_log!("Image conversion failed: {}", e);
                    Err(JsValue::from_str(&format!("Conversion failed: {}", e)))
                }
            }
        }
    }

    /// Get image information from file data using rust_core
    #[wasm_bindgen]
    pub fn get_image_info(&self, file_data: &[u8], format: &str) -> Result<JsValue, JsValue> {
        console_log!("Getting image info for {} format ({} bytes)", format, file_data.len());
        
        // Use the image crate to get real image information
        match image::load_from_memory(file_data) {
            Ok(img) => {
                let info = ImageInfo {
                    width: img.width(),
                    height: img.height(),
                    format: format.to_string(),
                    file_size: file_data.len(),
                };
                
                // Convert to JS object
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"width".into(), &JsValue::from(info.width))?;
                js_sys::Reflect::set(&obj, &"height".into(), &JsValue::from(info.height))?;
                js_sys::Reflect::set(&obj, &"format".into(), &JsValue::from_str(&info.format))?;
                js_sys::Reflect::set(&obj, &"size".into(), &JsValue::from(info.file_size))?;
                
                Ok(obj.into())
            },
            Err(e) => {
                console_log!("Failed to load image: {}", e);
                Err(JsValue::from_str(&format!("Failed to load image: {}", e)))
            }
        }
    }
}

// Mesh processing WASM bindings
#[wasm_bindgen]
pub struct MeshOptimizer {
    core_optimizer: CoreMeshOptimizer,
}

#[wasm_bindgen]
impl MeshOptimizer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MeshOptimizer {
        console_log!("Creating new MeshOptimizer with real rust_core backend");
        MeshOptimizer {
            core_optimizer: CoreMeshOptimizer::new(),
        }
    }

    /// Process a mesh file from browser File API using real optimization
    #[wasm_bindgen]
    pub async fn process_mesh_file(
        &self,
        file_data: &[u8],
        input_format: &str,
        config: &WasmOptConfig,
    ) -> Result<Vec<u8>, JsValue> {
        console_log!("Processing mesh: {} ({} bytes)", input_format, file_data.len());
        
        let core_config = CoreOptConfig::from(config);
        
        match self.core_optimizer.optimize(file_data, &core_config) {
            Ok(optimized_data) => {
                console_log!("Mesh optimized successfully: {} bytes -> {} bytes", 
                    file_data.len(), optimized_data.len());
                Ok(optimized_data)
            },
            Err(e) => {
                console_log!("Mesh optimization failed: {}", e);
                Err(JsValue::from_str(&format!("Mesh optimization failed: {}", e)))
            }
        }
    }
}

// File handling utilities
#[wasm_bindgen]
pub struct FileHandler;

#[wasm_bindgen]
impl FileHandler {
    /// Read a File object from the browser into bytes
    #[wasm_bindgen]
    pub async fn read_file_as_bytes(file: &File) -> Result<Vec<u8>, JsValue> {
        let file_reader = FileReader::new()?;
        
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let reader = file_reader.clone();
            
            let onload = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                if let Ok(result) = reader.result() {
                    if let Ok(array_buffer) = result.dyn_into::<js_sys::ArrayBuffer>() {
                        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                        let mut bytes = vec![0; uint8_array.length() as usize];
                        uint8_array.copy_to(&mut bytes);
                        resolve.call1(&JsValue::NULL, &JsValue::from(
                            js_sys::Uint8Array::from(&bytes[..])
                        )).unwrap();
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let onerror = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                reject.call1(&JsValue::NULL, &JsValue::from_str("Failed to read file")).unwrap();
            }) as Box<dyn FnMut(_)>);

            file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            file_reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            
            file_reader.read_as_array_buffer(file).unwrap();
            
            onload.forget();
            onerror.forget();
        });

        let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
        let uint8_array: js_sys::Uint8Array = result.into();
        
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);
        
        Ok(bytes)
    }

    /// Create a download URL for processed file data
    #[wasm_bindgen]
    pub fn create_download_url(data: &[u8], mime_type: &str) -> Result<String, JsValue> {
        let uint8_array = js_sys::Uint8Array::from(data);
        let array = js_sys::Array::new_with_length(1);
        array.set(0, uint8_array.into());
        
        let blob_options = web_sys::BlobPropertyBag::new();
        blob_options.set_type(mime_type);
        
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &array.into(),
            &blob_options,
        )?;
        
        web_sys::Url::create_object_url_with_blob(&blob)
    }
}

// Canvas utilities for image preview
#[wasm_bindgen]
pub struct CanvasRenderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl CanvasRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<CanvasRenderer, JsValue> {
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        Ok(CanvasRenderer { canvas, context })
    }

    /// Load and render an image from a blob URL
    #[wasm_bindgen]
    pub async fn load_and_render_image(&self, blob_url: &str) -> Result<(), JsValue> {
        let image = web_sys::HtmlImageElement::new()?;
        
        // Create a promise to wait for image loading
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let img = image.clone();
            let canvas = self.canvas.clone();
            let context = self.context.clone();
            
            let onload = Closure::wrap(Box::new(move || {
                let img_width = img.natural_width();
                let img_height = img.natural_height();
                
                // Resize canvas to fit image
                canvas.set_width(img_width);
                canvas.set_height(img_height);
                
                // Draw the image
                context.draw_image_with_html_image_element(&img, 0.0, 0.0).unwrap();
                
                resolve.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut()>);

            let onerror = Closure::wrap(Box::new(move || {
                reject.call1(&JsValue::NULL, &JsValue::from_str("Failed to load image")).unwrap();
            }) as Box<dyn FnMut()>);

            image.set_onload(Some(onload.as_ref().unchecked_ref()));
            image.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            
            onload.forget();
            onerror.forget();
        });

        image.set_src(blob_url);
        
        wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(())
    }

    /// Clear the canvas
    #[wasm_bindgen]
    pub fn clear(&self) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;
        self.context.clear_rect(0.0, 0.0, width, height);
    }
}

// Progress callback for long operations
#[wasm_bindgen]
pub struct ProgressCallback {
    callback: js_sys::Function,
}

#[wasm_bindgen]
impl ProgressCallback {
    #[wasm_bindgen(constructor)]
    pub fn new(callback: js_sys::Function) -> ProgressCallback {
        ProgressCallback { callback }
    }

    #[wasm_bindgen]
    pub fn update(&self, progress: f64, message: &str) -> Result<(), JsValue> {
        self.callback.call2(
            &JsValue::NULL,
            &JsValue::from(progress),
            &JsValue::from_str(message),
        )?;
        Ok(())
    }
}

// Utility functions
#[wasm_bindgen]
pub fn supported_image_formats() -> Vec<JsValue> {
    vec![
        JsValue::from_str("png"),
        JsValue::from_str("jpeg"),
        JsValue::from_str("jpg"),
        JsValue::from_str("gif"),
        JsValue::from_str("bmp"),
        JsValue::from_str("tiff"),
    ]
}

#[wasm_bindgen]
pub fn supported_mesh_formats() -> Vec<JsValue> {
    vec![
        JsValue::from_str("obj"),
        JsValue::from_str("ply"),
        JsValue::from_str("stl"),
        JsValue::from_str("gltf"),
        JsValue::from_str("glb"),
        JsValue::from_str("dae"),
        JsValue::from_str("fbx"),
        JsValue::from_str("usdz"),
    ]
}
