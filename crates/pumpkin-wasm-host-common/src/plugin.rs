use std::{any::Any, sync::Arc};
use thiserror::Error;

use crate::{concurrent_store::LegacyStore, scheduler::TaskScheduler};

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Could not create the Wasmtime engine")]
    EngineCreationFailed(#[source] wasmtime::Error),
    #[error("Could not set up the plugin linker")]
    LinkerSetupFailed(#[source] wasmtime::Error),
    #[error("The plugin has no versioned pumpkin:plugin/metadata export")]
    UnknownApiVersion(),
    #[error("This server cannot load plugins using Pumpkin plugin API {0}. It supports API {1}.")]
    UnsupportedApiVersion(String, &'static str),
    #[error("Plugin API 0.1 supports WASI 0.2 only, but the plugin imports `{0}`")]
    UnsupportedWasi(String),
    #[error(
        "The plugin's interface is incompatible with this server (expected Pumpkin plugin API {0})."
    )]
    ApiMismatch(PluginApiVersion, #[source] wasmtime::Error),
    #[error("Could not read the plugin file")]
    FileReadFailed(#[source] std::io::Error),
    #[error("Could not compile the plugin component")]
    ComponentNewFailed(#[source] wasmtime::Error),
    #[error("Could not serialize the compiled plugin cache")]
    ComponentCacheSerializeFailed(#[source] wasmtime::Error),
    #[error("Could not write the compiled plugin cache")]
    ComponentCacheWriteFailed(#[source] std::io::Error),
    #[error("Could not instantiate the plugin")]
    InstantiationFailed(#[source] wasmtime::Error),
    #[error("The plugin failed during init_plugin")]
    CallInitPluginFailed(#[source] wasmtime::Error),
    #[error("The plugin failed while providing its metadata")]
    CallGetMetadataFailed(#[source] wasmtime::Error),
    #[error("Could not resolve the plugin path")]
    PathResolutionFailed(#[source] std::io::Error),
    #[error("Could not create the Wasmtime cache")]
    CacheCreationFailed(#[source] wasmtime::Error),
}

pub const SUPPORTED_PLUGIN_APIS: [(&str, PluginApiVersion); 2] = [
    ("0.1", PluginApiVersion::V0_1),
    ("0.2", PluginApiVersion::V0_2),
];

impl PluginInitError {
    /// Classifies preparation errors after selecting the component's engine.
    #[must_use]
    pub fn preparation_failed(api_version: PluginApiVersion, error: wasmtime::Error) -> Self {
        if error.is::<wasmtime::OutOfMemory>() {
            Self::InstantiationFailed(error)
        } else {
            Self::ApiMismatch(api_version, error)
        }
    }

    /// Returns a hint when the error identifies a specific action.
    #[must_use]
    pub fn help(&self) -> Option<&'static str> {
        match self {
            Self::UnsupportedApiVersion(..) | Self::ApiMismatch(..) => Some(
                "Check the plugin's supported Pumpkin versions. Update Pumpkin or install a plugin release compatible with your server.",
            ),
            Self::FileReadFailed(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                Some("Check that the server can read the plugin file.")
            }
            Self::ComponentCacheWriteFailed(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied =>
            {
                Some("Check that the server can write to the plugin cache directory.")
            }
            _ => None,
        }
    }

    /// Returns the original Wasmtime error, if this variant contains one.
    #[must_use]
    pub const fn wasmtime_error(&self) -> Option<&wasmtime::Error> {
        match self {
            Self::EngineCreationFailed(error)
            | Self::LinkerSetupFailed(error)
            | Self::ApiMismatch(_, error)
            | Self::ComponentNewFailed(error)
            | Self::ComponentCacheSerializeFailed(error)
            | Self::InstantiationFailed(error)
            | Self::CallInitPluginFailed(error)
            | Self::CallGetMetadataFailed(error)
            | Self::CacheCreationFailed(error) => Some(error),
            _ => None,
        }
    }
}

/// What a plugin component declares at its top level, read without compiling it.
///
/// A compiled `Component` belongs to one `Engine`, and the engine depends on the plugin's API
/// version, so this has to be known first.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ComponentNames {
    /// The version of the exported `pumpkin:plugin/metadata` interface, such as `0.1.0`.
    pub metadata_version: Option<String>,
    /// The first imported WASI 0.3 interface, if any.
    pub wasi_p3_import: Option<String>,
    /// The guest binding language inferred from the WebAssembly `producers` custom section.
    pub binding_language: Option<String>,
}

/// Scans the top-level imports and exports of a component binary, along with any `producers`
/// custom sections used to identify the guest binding language.
///
/// # Errors
/// Returns [`PluginInitError::ComponentNewFailed`] when the bytes are not a valid component.
pub fn scan_component_names(wasm_bytes: &[u8]) -> Result<ComponentNames, PluginInitError> {
    let malformed = |error: wasmparser::BinaryReaderError| {
        PluginInitError::ComponentNewFailed(wasmtime::Error::new(error))
    };
    let mut names = ComponentNames::default();
    let mut language_detector = LanguageDetector::default();
    // Nested components and modules repeat the same section kinds; only depth 0 is the plugin.
    let mut depth = 0usize;
    for payload in wasmparser::Parser::new(0).parse_all(wasm_bytes) {
        match payload.map_err(malformed)? {
            wasmparser::Payload::ModuleSection { .. }
            | wasmparser::Payload::ComponentSection { .. } => depth += 1,
            wasmparser::Payload::End(_) => depth = depth.saturating_sub(1),
            wasmparser::Payload::ComponentImportSection(imports) if depth == 0 => {
                for import in imports {
                    let name = import.map_err(malformed)?.name.full_name();
                    if names.wasi_p3_import.is_none() && is_wasi_p3_import(&name) {
                        names.wasi_p3_import = Some(name.into_owned());
                    }
                }
            }
            wasmparser::Payload::ComponentExportSection(exports) if depth == 0 => {
                for export in exports {
                    let name = export.map_err(malformed)?.name.full_name();
                    if let Some(version) = name.strip_prefix("pumpkin:plugin/metadata@") {
                        names.metadata_version = Some(version.to_owned());
                    }
                }
            }
            wasmparser::Payload::CustomSection(section) => {
                if let wasmparser::KnownCustom::Producers(producers) = section.as_known() {
                    language_detector.observe_section(producers);
                }
            }
            _ => {}
        }
    }
    names.binding_language = language_detector.finish();
    Ok(names)
}

#[derive(Default)]
struct LanguageDetector {
    best: Option<(u8, String)>,
    saw_cpp: bool,
}

impl LanguageDetector {
    fn observe_section(&mut self, producers: wasmparser::ProducersSectionReader<'_>) {
        let mut section_has_rust = false;
        let mut section_candidates: Vec<(u8, String)> = Vec::new();

        for field in producers.into_iter().flatten() {
            for value in field.values.into_iter().flatten() {
                match field.name {
                    "processed-by" | "sdk" => {
                        if let Some((priority, lang)) = Self::classify_tool(value.name) {
                            if lang == "rust" {
                                section_has_rust = true;
                            } else if lang == "cpp" {
                                self.saw_cpp = true;
                            }
                            section_candidates.push((priority, lang.to_owned()));
                        }
                    }
                    "language" => {
                        if let Some((priority, lang)) = Self::classify_language(value.name) {
                            if lang == "rust" {
                                section_has_rust = true;
                            } else if lang == "cpp" {
                                self.saw_cpp = true;
                            }
                            section_candidates.push((priority, lang));
                        }
                    }
                    _ => {}
                }
            }
        }

        for (priority, lang) in section_candidates {
            // Rust on WASI links `wasi-libc` (compiled in C) into the same core module, so a C entry
            // inside a section that also declares Rust is from libc rather than a C guest plugin.
            if section_has_rust && lang == "c" && priority < 3 {
                continue;
            }
            if self.best.as_ref().is_none_or(|(p, _)| priority > *p) {
                self.best = Some((priority, lang));
            }
        }
    }

    fn classify_tool(tool: &str) -> Option<(u8, &'static str)> {
        let lower = tool.trim().to_ascii_lowercase();
        if lower.contains("componentize-py") {
            Some((4, "python"))
        } else if lower.contains("componentize-js")
            || lower.contains("componentizejs")
            || lower == "jco"
            || lower.contains("starlingmonkey")
        {
            Some((4, "javascript"))
        } else if lower.contains("componentize-dotnet") || lower.contains("wit-bindgen-csharp") {
            Some((4, "csharp"))
        } else if lower.contains("wit-bindgen-go") || lower == "tinygo" {
            Some((4, "go"))
        } else if lower.contains("wit-bindgen-moonbit") {
            Some((4, "moonbit"))
        } else if lower.contains("wit-bindgen-teavm-java") {
            Some((4, "java"))
        } else if lower.contains("wit-bindgen-c") {
            Some((3, "c"))
        } else if lower.contains("wit-bindgen-rust") || lower.contains("cargo-component") {
            Some((3, "rust"))
        } else if lower == "clang++" {
            Some((2, "cpp"))
        } else if lower == "clang" {
            Some((1, "c"))
        } else if lower == "rustc" {
            Some((1, "rust"))
        } else {
            None
        }
    }

    fn classify_language(language: &str) -> Option<(u8, String)> {
        let lower = language.trim().to_ascii_lowercase();
        match lower.as_str() {
            "" | "wat" => None,
            "rust" => Some((1, "rust".to_owned())),
            "c_plus_plus" | "c++" | "cpp" => Some((2, "cpp".to_owned())),
            "c" => Some((2, "c".to_owned())),
            "c#" | "csharp" => Some((4, "csharp".to_owned())),
            "javascript" | "js" => Some((4, "javascript".to_owned())),
            "typescript" | "ts" => Some((4, "typescript".to_owned())),
            "python" | "py" => Some((4, "python".to_owned())),
            "go" | "golang" => Some((4, "go".to_owned())),
            other => Some((4, other.to_owned())),
        }
    }

    fn finish(self) -> Option<String> {
        self.best.map(|(_, lang)| {
            if lang == "c" && self.saw_cpp {
                "cpp".to_owned()
            } else {
                lang
            }
        })
    }
}

fn is_wasi_p3_import(name: &str) -> bool {
    name.starts_with("wasi:")
        && name
            .rsplit_once('@')
            .is_some_and(|(_, version)| version.starts_with("0.3."))
}

#[derive(Copy, Clone, Debug)]
pub enum PluginApiVersion {
    V0_1,
    V0_2,
}

impl std::fmt::Display for PluginApiVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::V0_1 => "0.1.0",
            Self::V0_2 => "0.2.0",
        })
    }
}

pub struct WasmPlugin {
    pub name: String,
    pub version: String,
    /// The guest exports of the plugin, as generated for `api_version`.
    pub plugin_instance: Arc<dyn Any + Send + Sync>,
    pub api_version: PluginApiVersion,
    pub binding_language: Option<String>,
    pub store: LegacyStore,
    pub task_scheduler: Arc<TaskScheduler>,
}

impl WasmPlugin {
    /// Logs the skipped callback and returns `true` if the plugin store is stopped.
    #[must_use]
    pub fn report_stopped(&self, operation: &str) -> bool {
        if !self.store.state().is_terminal() {
            return false;
        }
        tracing::error!(
            "Plugin \"{}\" ({}) is stopped; skipped {}.",
            pumpkin_core::plugin::error::inline(&self.name),
            pumpkin_core::plugin::error::inline(&self.version),
            pumpkin_core::plugin::error::inline(operation)
        );
        true
    }

    /// Logs a callback failure at ERROR and its original Wasmtime details at DEBUG.
    pub fn report_failure(&self, operation: &str, failure: &wasmtime::Error) {
        use pumpkin_core::plugin::error;
        let headline = format!(
            "Plugin \"{}\" ({}) failed during {}.",
            error::inline(&self.name),
            error::inline(&self.version),
            error::inline(operation)
        );
        let stopped = self.store.state().is_terminal()
            || pumpkin_plugin_runtime::is_guest_call_failure(failure);
        let help = crate::error::help(failure)
            .or_else(|| stopped.then_some("Reload the plugin or restart the server to retry."));
        let text = crate::error::message(
            &headline,
            &crate::error::causes(failure),
            stopped.then_some("The plugin has stopped."),
            help,
            Some(failure),
        );
        tracing::error!("{text}");
        error::debug_details(&headline, pumpkin_plugin_runtime::guest_call_error(failure));
    }

    /// The guest exports of this plugin. Each API version's host only ever sees plugins of its own version.
    ///
    /// # Panics
    /// Panics when `T` is not the type generated for this plugin's API version.
    #[must_use]
    pub fn instance<T: 'static>(&self) -> &T {
        self.plugin_instance
            .downcast_ref()
            .unwrap_or_else(|| panic!("Unexpected plugin version in host path."))
    }
}

impl Drop for WasmPlugin {
    fn drop(&mut self) {
        self.store.discard();
    }
}

#[cfg(test)]
mod tests {
    use super::{PluginApiVersion, PluginInitError, is_wasi_p3_import, scan_component_names};
    use std::io::{Error, ErrorKind};
    use wasm_encoder::{
        ComponentBuilder, ComponentTypeRef, InstanceType, Module, ProducersField, ProducersSection,
    };

    #[test]
    fn help_is_limited_to_actionable_typed_failures() {
        assert!(
            PluginInitError::UnsupportedApiVersion("0.3.0".into(), "0.1 and 0.2")
                .help()
                .is_some()
        );
        assert!(
            PluginInitError::ApiMismatch(
                PluginApiVersion::V0_1,
                wasmtime::Error::msg("type mismatch")
            )
            .help()
            .is_some()
        );
        for kind in [ErrorKind::PermissionDenied, ErrorKind::NotFound] {
            let expected = kind == ErrorKind::PermissionDenied;
            assert_eq!(
                PluginInitError::FileReadFailed(Error::from(kind))
                    .help()
                    .is_some(),
                expected
            );
            assert_eq!(
                PluginInitError::ComponentCacheWriteFailed(Error::from(kind))
                    .help()
                    .is_some(),
                expected
            );
        }
        for error in [
            PluginInitError::EngineCreationFailed(wasmtime::Error::msg("failed")),
            PluginInitError::LinkerSetupFailed(wasmtime::Error::msg("failed")),
            PluginInitError::UnknownApiVersion(),
            PluginInitError::ComponentNewFailed(wasmtime::Error::msg("Permission denied")),
            PluginInitError::ComponentCacheSerializeFailed(wasmtime::Error::msg("failed")),
            PluginInitError::InstantiationFailed(wasmtime::Error::msg("failed")),
            PluginInitError::CallInitPluginFailed(wasmtime::Error::msg("failed")),
            PluginInitError::CallGetMetadataFailed(wasmtime::Error::msg("failed")),
            PluginInitError::PathResolutionFailed(Error::from(ErrorKind::PermissionDenied)),
            PluginInitError::CacheCreationFailed(wasmtime::Error::msg("failed")),
        ] {
            assert!(error.help().is_none(), "{error}");
        }
    }

    #[test]
    fn preparation_resource_failures_do_not_suggest_api_changes() {
        let source = wasmtime::Error::from(wasmtime::OutOfMemory::new(4096));
        let failure = PluginInitError::preparation_failed(PluginApiVersion::V0_1, source);
        assert!(matches!(failure, PluginInitError::InstantiationFailed(_)));
        assert!(failure.help().is_none());
        assert!(
            failure
                .wasmtime_error()
                .unwrap()
                .is::<wasmtime::OutOfMemory>()
        );
    }

    fn component_importing(names: &[&str]) -> Vec<u8> {
        let mut component = ComponentBuilder::default();
        for name in names {
            let ty = component.type_instance(None, &InstanceType::new());
            component.import(*name, ComponentTypeRef::Instance(ty));
        }
        component.finish()
    }

    fn module_with_producers(languages: &[&str], tools: &[&str]) -> Module {
        let mut producers = ProducersSection::new();
        if !languages.is_empty() {
            let mut field = ProducersField::new();
            for &lang in languages {
                field.value(lang, "1.0");
            }
            producers.field("language", &field);
        }
        if !tools.is_empty() {
            let mut field = ProducersField::new();
            for &tool in tools {
                field.value(tool, "1.0");
            }
            producers.field("processed-by", &field);
        }
        let mut module = Module::new();
        module.section(&producers);
        module
    }

    fn component_with_modules(modules: &[Module]) -> Vec<u8> {
        let mut component = ComponentBuilder::default();
        for module in modules {
            component.core_module(None, module);
        }
        component.finish()
    }

    #[test]
    fn wasi_p3_imports_are_recognised_by_version() {
        assert!(is_wasi_p3_import("wasi:random/random@0.3.0"));
        assert!(is_wasi_p3_import("wasi:cli/stdout@0.3.0-rc-2026-03-15"));
        assert!(!is_wasi_p3_import("wasi:random/random@0.2.6"));
        assert!(!is_wasi_p3_import("wasi:random/random"));
        assert!(!is_wasi_p3_import("pumpkin:plugin/context@0.3.0"));
    }

    #[test]
    fn scan_reports_the_first_wasi_p3_import() {
        let bytes = component_importing(&["wasi:random/random@0.2.6", "wasi:random/random@0.3.0"]);
        let names = scan_component_names(&bytes).expect("scan component");
        assert_eq!(
            names.wasi_p3_import.as_deref(),
            Some("wasi:random/random@0.3.0")
        );
        assert_eq!(names.metadata_version, None);
        assert_eq!(names.binding_language, None);
    }

    #[test]
    fn scan_detects_binding_languages_from_producers_sections() {
        // Rust module linking wasi-libc (which adds C alongside Rust in the same module)
        let rust_module =
            module_with_producers(&["C", "Rust"], &["clang", "rustc", "wit-bindgen-rust"]);
        let bytes = component_with_modules(&[rust_module]);
        assert_eq!(
            scan_component_names(&bytes)
                .unwrap()
                .binding_language
                .as_deref(),
            Some("rust")
        );

        // Go guest module paired with a Rust WASI preview1 adapter module
        let go_module = module_with_producers(&["Go"], &["tinygo", "wit-bindgen-go"]);
        let adapter_module = module_with_producers(&["C", "Rust"], &["rustc"]);
        let bytes = component_with_modules(&[go_module, adapter_module.clone()]);
        assert_eq!(
            scan_component_names(&bytes)
                .unwrap()
                .binding_language
                .as_deref(),
            Some("go")
        );

        // Python guest componentized with CPython (C) and a Rust adapter
        let py_module = module_with_producers(&["C"], &["clang", "componentize-py"]);
        let bytes = component_with_modules(&[py_module, adapter_module.clone()]);
        assert_eq!(
            scan_component_names(&bytes)
                .unwrap()
                .binding_language
                .as_deref(),
            Some("python")
        );

        // JavaScript guest componentized with StarlingMonkey (C++) and a Rust adapter
        let js_module = module_with_producers(&["C_plus_plus"], &["ComponentizeJS"]);
        let bytes = component_with_modules(&[js_module, adapter_module.clone()]);
        assert_eq!(
            scan_component_names(&bytes)
                .unwrap()
                .binding_language
                .as_deref(),
            Some("javascript")
        );

        // C++ guest using wit-bindgen-c paired with a Rust WASI adapter
        let cpp_module = module_with_producers(&["C_plus_plus"], &["clang++", "wit-bindgen-c"]);
        let bytes = component_with_modules(&[cpp_module, adapter_module]);
        assert_eq!(
            scan_component_names(&bytes)
                .unwrap()
                .binding_language
                .as_deref(),
            Some("cpp")
        );
    }

    #[test]
    fn scan_rejects_bytes_that_are_not_a_component() {
        assert!(scan_component_names(b"not wasm").is_err());
    }
}
