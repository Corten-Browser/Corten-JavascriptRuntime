use std::collections::HashMap;

/// WebAssembly module (compiled WASM)
#[derive(Debug)]
pub struct WasmModule {
    bytes: Vec<u8>,
    exports: Vec<ExportDescriptor>,
    imports: Vec<ImportDescriptor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExportDescriptor {
    pub name: String,
    pub kind: ExportKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportKind {
    Function,
    Memory,
    Table,
    Global,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportDescriptor {
    pub module: String,
    pub name: String,
    pub kind: ExportKind,
}

impl WasmModule {
    /// Compile WASM bytes into module
    pub fn compile(bytes: &[u8]) -> Result<Self, String> {
        // Validate WASM magic number
        if bytes.len() < 8 {
            return Err("Invalid WASM: too short".to_string());
        }
        if &bytes[0..4] != b"\x00asm" {
            return Err("Invalid WASM: missing magic number".to_string());
        }

        // Parse version (1)
        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        if version != 1 {
            return Err(format!("Unsupported WASM version: {}", version));
        }

        // Parse exports and imports (simplified)
        let exports = Self::parse_exports(bytes);
        let imports = Self::parse_imports(bytes);

        Ok(Self {
            bytes: bytes.to_vec(),
            exports,
            imports,
        })
    }

    fn parse_exports(_bytes: &[u8]) -> Vec<ExportDescriptor> {
        // Simplified: return empty for now
        vec![]
    }

    fn parse_imports(_bytes: &[u8]) -> Vec<ImportDescriptor> {
        vec![]
    }

    pub fn exports(&self) -> &[ExportDescriptor] {
        &self.exports
    }

    pub fn imports(&self) -> &[ImportDescriptor] {
        &self.imports
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// WebAssembly instance (instantiated module)
pub struct WasmInstance {
    module: WasmModule,
    memory: Option<WasmMemory>,
    exports: HashMap<String, WasmExport>,
}

pub enum WasmExport {
    Function(WasmFunction),
    Memory(WasmMemory),
    Global(WasmGlobal),
}

pub struct WasmFunction {
    name: String,
    params: Vec<WasmType>,
    results: Vec<WasmType>,
}

impl WasmFunction {
    pub fn new(name: String, params: Vec<WasmType>, results: Vec<WasmType>) -> Self {
        Self {
            name,
            params,
            results,
        }
    }

    pub fn call(&self, _args: &[WasmValue]) -> Result<Vec<WasmValue>, String> {
        // Simplified: return empty result
        Ok(vec![])
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[WasmType] {
        &self.params
    }

    pub fn results(&self) -> &[WasmType] {
        &self.results
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// WebAssembly linear memory
pub struct WasmMemory {
    data: Vec<u8>,
    max_pages: Option<u32>,
}

impl WasmMemory {
    pub fn new(initial_pages: u32, max_pages: Option<u32>) -> Self {
        let size = (initial_pages as usize) * 65536; // 64KB per page
        Self {
            data: vec![0; size],
            max_pages,
        }
    }

    pub fn grow(&mut self, pages: u32) -> i32 {
        let current_pages = (self.data.len() / 65536) as u32;
        let new_pages = current_pages + pages;

        if let Some(max) = self.max_pages {
            if new_pages > max {
                return -1; // Growth failed
            }
        }

        let new_size = (new_pages as usize) * 65536;
        self.data.resize(new_size, 0);
        current_pages as i32
    }

    pub fn buffer(&self) -> &[u8] {
        &self.data
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn page_count(&self) -> u32 {
        (self.data.len() / 65536) as u32
    }

    pub fn byte_length(&self) -> usize {
        self.data.len()
    }

    pub fn max_pages(&self) -> Option<u32> {
        self.max_pages
    }
}

impl Clone for WasmMemory {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            max_pages: self.max_pages,
        }
    }
}

pub struct WasmGlobal {
    value: WasmValue,
    mutable: bool,
}

impl WasmGlobal {
    pub fn new(value: WasmValue, mutable: bool) -> Self {
        Self { value, mutable }
    }

    pub fn value(&self) -> &WasmValue {
        &self.value
    }

    pub fn set_value(&mut self, value: WasmValue) -> Result<(), String> {
        if !self.mutable {
            return Err("Cannot modify immutable global".to_string());
        }
        self.value = value;
        Ok(())
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable
    }
}

impl WasmInstance {
    pub fn new(
        module: WasmModule,
        _imports: HashMap<String, WasmExport>,
    ) -> Result<Self, String> {
        // Create default memory
        let memory = Some(WasmMemory::new(1, None));

        Ok(Self {
            module,
            memory,
            exports: HashMap::new(),
        })
    }

    pub fn get_export(&self, name: &str) -> Option<&WasmExport> {
        self.exports.get(name)
    }

    pub fn memory(&self) -> Option<&WasmMemory> {
        self.memory.as_ref()
    }

    pub fn memory_mut(&mut self) -> Option<&mut WasmMemory> {
        self.memory.as_mut()
    }

    pub fn module(&self) -> &WasmModule {
        &self.module
    }

    pub fn add_export(&mut self, name: String, export: WasmExport) {
        self.exports.insert(name, export);
    }
}

/// Main WebAssembly API object
pub struct WebAssembly;

impl WebAssembly {
    pub fn compile(bytes: &[u8]) -> Result<WasmModule, String> {
        WasmModule::compile(bytes)
    }

    pub fn instantiate(
        module: WasmModule,
        imports: HashMap<String, WasmExport>,
    ) -> Result<WasmInstance, String> {
        WasmInstance::new(module, imports)
    }

    pub fn validate(bytes: &[u8]) -> bool {
        bytes.len() >= 8 && &bytes[0..4] == b"\x00asm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid minimal WASM module (magic number + version)
    fn minimal_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // Magic: \0asm
            0x01, 0x00, 0x00, 0x00, // Version: 1
        ]
    }

    #[test]
    fn test_wasm_module_compile_valid() {
        let bytes = minimal_wasm();
        let result = WasmModule::compile(&bytes);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.bytes(), &bytes);
    }

    #[test]
    fn test_wasm_module_compile_too_short() {
        let bytes = vec![0x00, 0x61, 0x73, 0x6d];
        let result = WasmModule::compile(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid WASM: too short");
    }

    #[test]
    fn test_wasm_module_compile_invalid_magic() {
        let bytes = vec![
            0xFF, 0xFF, 0xFF, 0xFF, // Wrong magic
            0x01, 0x00, 0x00, 0x00,
        ];
        let result = WasmModule::compile(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid WASM: missing magic number");
    }

    #[test]
    fn test_wasm_module_compile_wrong_version() {
        let bytes = vec![
            0x00, 0x61, 0x73, 0x6d, // Magic: \0asm
            0x02, 0x00, 0x00, 0x00, // Version: 2 (unsupported)
        ];
        let result = WasmModule::compile(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported WASM version"));
    }

    #[test]
    fn test_wasm_module_exports_empty() {
        let bytes = minimal_wasm();
        let module = WasmModule::compile(&bytes).unwrap();
        assert!(module.exports().is_empty());
    }

    #[test]
    fn test_wasm_module_imports_empty() {
        let bytes = minimal_wasm();
        let module = WasmModule::compile(&bytes).unwrap();
        assert!(module.imports().is_empty());
    }

    #[test]
    fn test_webassembly_validate_valid() {
        let bytes = minimal_wasm();
        assert!(WebAssembly::validate(&bytes));
    }

    #[test]
    fn test_webassembly_validate_invalid_magic() {
        let bytes = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00];
        assert!(!WebAssembly::validate(&bytes));
    }

    #[test]
    fn test_webassembly_validate_too_short() {
        let bytes = vec![0x00, 0x61, 0x73, 0x6d];
        assert!(!WebAssembly::validate(&bytes));
    }

    #[test]
    fn test_webassembly_compile() {
        let bytes = minimal_wasm();
        let result = WebAssembly::compile(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_webassembly_instantiate() {
        let bytes = minimal_wasm();
        let module = WebAssembly::compile(&bytes).unwrap();
        let imports = HashMap::new();
        let result = WebAssembly::instantiate(module, imports);
        assert!(result.is_ok());
        let instance = result.unwrap();
        assert!(instance.memory().is_some());
    }

    #[test]
    fn test_wasm_instance_has_default_memory() {
        let bytes = minimal_wasm();
        let module = WebAssembly::compile(&bytes).unwrap();
        let instance = WebAssembly::instantiate(module, HashMap::new()).unwrap();
        let memory = instance.memory().unwrap();
        assert_eq!(memory.page_count(), 1);
        assert_eq!(memory.byte_length(), 65536);
    }

    #[test]
    fn test_wasm_instance_get_export_not_found() {
        let bytes = minimal_wasm();
        let module = WebAssembly::compile(&bytes).unwrap();
        let instance = WebAssembly::instantiate(module, HashMap::new()).unwrap();
        assert!(instance.get_export("nonexistent").is_none());
    }

    #[test]
    fn test_wasm_memory_new() {
        let memory = WasmMemory::new(2, Some(10));
        assert_eq!(memory.page_count(), 2);
        assert_eq!(memory.byte_length(), 2 * 65536);
        assert_eq!(memory.max_pages(), Some(10));
    }

    #[test]
    fn test_wasm_memory_grow_success() {
        let mut memory = WasmMemory::new(1, Some(5));
        let old_pages = memory.grow(2);
        assert_eq!(old_pages, 1);
        assert_eq!(memory.page_count(), 3);
        assert_eq!(memory.byte_length(), 3 * 65536);
    }

    #[test]
    fn test_wasm_memory_grow_failure_exceeds_max() {
        let mut memory = WasmMemory::new(1, Some(3));
        let result = memory.grow(5);
        assert_eq!(result, -1);
        assert_eq!(memory.page_count(), 1); // Unchanged
    }

    #[test]
    fn test_wasm_memory_grow_no_max() {
        let mut memory = WasmMemory::new(1, None);
        let old_pages = memory.grow(10);
        assert_eq!(old_pages, 1);
        assert_eq!(memory.page_count(), 11);
    }

    #[test]
    fn test_wasm_memory_buffer_read_write() {
        let mut memory = WasmMemory::new(1, None);
        let buffer = memory.buffer_mut();
        buffer[0] = 0xFF;
        buffer[100] = 0xAB;
        buffer[65535] = 0xCD;

        let read_buffer = memory.buffer();
        assert_eq!(read_buffer[0], 0xFF);
        assert_eq!(read_buffer[100], 0xAB);
        assert_eq!(read_buffer[65535], 0xCD);
    }

    #[test]
    fn test_wasm_memory_initial_zeros() {
        let memory = WasmMemory::new(1, None);
        let buffer = memory.buffer();
        for byte in buffer.iter() {
            assert_eq!(*byte, 0);
        }
    }

    #[test]
    fn test_wasm_memory_clone() {
        let mut memory = WasmMemory::new(1, Some(10));
        memory.buffer_mut()[0] = 0x42;
        let cloned = memory.clone();
        assert_eq!(cloned.page_count(), memory.page_count());
        assert_eq!(cloned.max_pages(), memory.max_pages());
        assert_eq!(cloned.buffer()[0], 0x42);
    }

    #[test]
    fn test_wasm_function_new() {
        let func = WasmFunction::new(
            "add".to_string(),
            vec![WasmType::I32, WasmType::I32],
            vec![WasmType::I32],
        );
        assert_eq!(func.name(), "add");
        assert_eq!(func.params(), &[WasmType::I32, WasmType::I32]);
        assert_eq!(func.results(), &[WasmType::I32]);
    }

    #[test]
    fn test_wasm_function_call() {
        let func = WasmFunction::new(
            "add".to_string(),
            vec![WasmType::I32, WasmType::I32],
            vec![WasmType::I32],
        );
        let args = vec![WasmValue::I32(10), WasmValue::I32(20)];
        let result = func.call(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wasm_global_immutable() {
        let global = WasmGlobal::new(WasmValue::I32(42), false);
        assert_eq!(global.value(), &WasmValue::I32(42));
        assert!(!global.is_mutable());
    }

    #[test]
    fn test_wasm_global_mutable() {
        let mut global = WasmGlobal::new(WasmValue::I32(42), true);
        assert!(global.is_mutable());
        let result = global.set_value(WasmValue::I32(100));
        assert!(result.is_ok());
        assert_eq!(global.value(), &WasmValue::I32(100));
    }

    #[test]
    fn test_wasm_global_set_immutable_fails() {
        let mut global = WasmGlobal::new(WasmValue::I32(42), false);
        let result = global.set_value(WasmValue::I32(100));
        assert!(result.is_err());
        assert_eq!(global.value(), &WasmValue::I32(42));
    }

    #[test]
    fn test_wasm_value_types() {
        let i32_val = WasmValue::I32(-100);
        let i64_val = WasmValue::I64(9999999999);
        let f32_val = WasmValue::F32(3.14);
        let f64_val = WasmValue::F64(2.71828);

        assert_eq!(i32_val, WasmValue::I32(-100));
        assert_eq!(i64_val, WasmValue::I64(9999999999));
        assert_eq!(f32_val, WasmValue::F32(3.14));
        assert_eq!(f64_val, WasmValue::F64(2.71828));
    }

    #[test]
    fn test_export_descriptor() {
        let desc = ExportDescriptor {
            name: "memory".to_string(),
            kind: ExportKind::Memory,
        };
        assert_eq!(desc.name, "memory");
        assert_eq!(desc.kind, ExportKind::Memory);
    }

    #[test]
    fn test_import_descriptor() {
        let desc = ImportDescriptor {
            module: "env".to_string(),
            name: "print".to_string(),
            kind: ExportKind::Function,
        };
        assert_eq!(desc.module, "env");
        assert_eq!(desc.name, "print");
        assert_eq!(desc.kind, ExportKind::Function);
    }

    #[test]
    fn test_wasm_instance_add_export() {
        let bytes = minimal_wasm();
        let module = WebAssembly::compile(&bytes).unwrap();
        let mut instance = WebAssembly::instantiate(module, HashMap::new()).unwrap();

        let func =
            WasmFunction::new("test".to_string(), vec![WasmType::I32], vec![WasmType::I32]);
        instance.add_export("test".to_string(), WasmExport::Function(func));

        assert!(instance.get_export("test").is_some());
    }

    #[test]
    fn test_wasm_instance_memory_mut() {
        let bytes = minimal_wasm();
        let module = WebAssembly::compile(&bytes).unwrap();
        let mut instance = WebAssembly::instantiate(module, HashMap::new()).unwrap();

        {
            let memory = instance.memory_mut().unwrap();
            memory.buffer_mut()[0] = 0xDE;
        }

        let memory = instance.memory().unwrap();
        assert_eq!(memory.buffer()[0], 0xDE);
    }

    #[test]
    fn test_wasm_instance_module_reference() {
        let bytes = minimal_wasm();
        let module = WebAssembly::compile(&bytes).unwrap();
        let instance = WebAssembly::instantiate(module, HashMap::new()).unwrap();
        assert_eq!(instance.module().bytes(), &minimal_wasm());
    }

    #[test]
    fn test_export_kind_variants() {
        assert_eq!(ExportKind::Function, ExportKind::Function);
        assert_eq!(ExportKind::Memory, ExportKind::Memory);
        assert_eq!(ExportKind::Table, ExportKind::Table);
        assert_eq!(ExportKind::Global, ExportKind::Global);
        assert_ne!(ExportKind::Function, ExportKind::Memory);
    }

    #[test]
    fn test_wasm_type_variants() {
        assert_eq!(WasmType::I32, WasmType::I32);
        assert_eq!(WasmType::I64, WasmType::I64);
        assert_eq!(WasmType::F32, WasmType::F32);
        assert_eq!(WasmType::F64, WasmType::F64);
        assert_ne!(WasmType::I32, WasmType::I64);
    }
}
