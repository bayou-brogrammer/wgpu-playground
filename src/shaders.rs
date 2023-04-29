use crate::dsl;
use glass::wgpu;
use regex::Regex;

#[derive(Default, Debug)]
pub struct ShaderImports {
    imports: Vec<String>,
    import_path: Option<String>,
}

pub struct ShaderImportProcessor {
    import_custom_path_regex: Regex,
    define_import_path_regex: Regex,
}

impl Default for ShaderImportProcessor {
    fn default() -> Self {
        Self {
            import_custom_path_regex: Regex::new(r"^\s*#\s*import\s+(.+)").unwrap(),
            define_import_path_regex: Regex::new(r"^\s*#\s*define_import_path\s+(.+)").unwrap(),
        }
    }
}

impl ShaderImportProcessor {
    pub fn load_shader(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        label: Option<&str>,
    ) -> std::io::Result<wgpu::ShaderModule> {
        let shader = self.load_shader_inner(shader_path)?;

        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&shader)),
        }))
    }

    pub fn load_shader_with_dsl(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        dsl: &dsl::Statement,
        label: Option<&str>,
    ) -> std::io::Result<wgpu::ShaderModule> {
        let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
        let shader_contents = self.load_shader_inner(shader_path)?;

        // Replace base shader with the shader rules
        let shader_rules = dsl.to_shader();
        let shader = shader_contents.replace("{PLACEHOLDER}", &shader_rules);

        if std::env::var("DEBUG_SHADER").is_ok() {
            std::fs::write(format!("{root}/{shader_path}.debug.wgsl"), shader.clone())
                .expect("Failed to write shader file");
        }

        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&shader)),
        }))
    }

    pub fn get_imports_from_str(&self, shader: &str) -> ShaderImports {
        let mut shader_imports = ShaderImports::default();
        for line in shader.lines() {
            if let Some(cap) = self.import_custom_path_regex.captures(line) {
                let import = cap.get(1).unwrap();
                shader_imports.imports.push(import.as_str().to_string());
            } else if let Some(cap) = self.define_import_path_regex.captures(line) {
                let path = cap.get(1).unwrap();
                shader_imports.import_path = Some(path.as_str().to_string());
            }
        }

        shader_imports
    }

    fn load_shader_inner(&self, shader_path: &str) -> std::io::Result<String> {
        let root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
        let mut shader_contents = match std::fs::read_to_string(format!("{root}/{shader_path}")) {
            Ok(contents) => contents,
            Err(err) => {
                log::error!("Failed to read shader file: {}", shader_path);
                return Err(err);
            }
        };

        // Replace all imports with the contents of the imported file
        let imports = self.get_imports_from_str(shader_contents.as_str());
        imports.imports.iter().for_each(|import| {
            let import_path = match &imports.import_path {
                Some(path) => format!("{root}/{path}"),
                None => root.to_string(),
            };

            let import_contents =
                match std::fs::read_to_string(format!("{}/{}", import_path, import)) {
                    Ok(contents) => contents,
                    Err(err) => {
                        log::error!("Failed to read import file: {} {}", import, err);
                        std::process::exit(1);
                    }
                };

            let import_string: String = format!("#import {import}");
            shader_contents = shader_contents.replace(&import_string, import_contents.as_str());
        });

        Ok(shader_contents)
    }
}
