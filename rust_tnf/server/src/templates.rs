use custom_error::custom_error;
use lazy_static::lazy_static;
use serde::Serialize;
use tera::{compile_templates, Tera};

struct Templates {
    tera: Tera,
    css: String,
}

custom_error! {pub TemplatesError
    Io{source: std::io::Error}     = "fs error",
    Tera{source: tera::Error}      = "tera error",
    Rsass{inner: rsass::Error}     = "rsass error",
    Utf8{source: std::string::FromUtf8Error} = "string conversion error"
}

impl Templates {
    fn new() -> Self {
        let css = Self::compile_css().unwrap();
        let tera = compile_templates!("web/templates/**/*");
        let templates = Templates { tera, css };
        templates.write_css().unwrap();
        templates
    }
    fn remake(&mut self) -> Result<(), TemplatesError> {
        self.css = Self::compile_css()?;
        self.tera.full_reload()?;
        self.write_css()?;
        Ok(())
    }
    fn write_css(&self) -> std::io::Result<()> {
        std::fs::write("web/static/charsheet.css", &self.css)
    }
    fn compile_css() -> Result<String, TemplatesError> {
        let vec = rsass::compile_scss_file(
            "web/styles/charsheet.scss".as_ref(),
            rsass::OutputStyle::Expanded,
        )
        .map_err(|err| TemplatesError::Rsass { inner: err })?;
        let string = String::from_utf8(vec)?;
        //OutputStyle::Compressed
        Ok(string)
    }
}

#[cfg(not(debug_assertions))]
lazy_static! {
    static ref TEMPLATES: Templates = {
        let mut templates = Templates::new();
        templates
    };
}
#[cfg(debug_assertions)]
lazy_static! {
    static ref TEMPLATES: std::sync::Mutex<Templates> = {
        let mut templates = Templates::new();
        std::sync::Mutex::new(templates)
    };
}

#[cfg(not(debug_assertions))]
pub fn render<T: Serialize>(template: &str, data: &T) -> Result<String, TemplatesError> {
    Ok(TEMPLATES.tera.render(template, data)?)
}
#[cfg(debug_assertions)]
pub fn render<T: Serialize>(template: &str, data: &T) -> Result<String, TemplatesError> {
    let mut templates = TEMPLATES.lock().unwrap();
    templates.remake()?;
    Ok(templates.tera.render(template, data)?)
}

pub fn init() {
    let _ = *TEMPLATES;
}
