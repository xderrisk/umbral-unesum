use std::{fs, path::Path, process::Command};

fn compile_blueprint(input: &Path, output: &Path) {
    let status = Command::new("blueprint-compiler")
        .args([
            "compile",
            "--output",
            output.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .status()
        .expect("No se pudo ejecutar blueprint-compiler");
    assert!(status.success());
}

fn main() {
    println!("cargo:rerun-if-changed=src/ui");
    let ui_src = Path::new("src/ui");
    let ui_out = Path::new("data/ui");
    let gresource_xml_path = "data/resources.gresource.xml";
    fs::create_dir_all(ui_out).unwrap();
    if ui_out.exists() {
        let _ = fs::remove_dir_all(ui_out);
        fs::create_dir_all(ui_out).unwrap();
    }
    let mut ui_files = std::collections::BTreeSet::new();
    for entry in fs::read_dir(ui_src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("blp") {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            let output = ui_out.join(format!("{}.ui", file_name));
            println!("cargo:rerun-if-changed={}", path.display());
            compile_blueprint(&path, &output);
            ui_files.insert(format!("ui/{}.ui", file_name));
        }
    }
    let mut xml_content = String::new();
    xml_content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml_content.push_str("<gresources>\n");
    xml_content.push_str("  <gresource prefix=\"/edu/unesum/umbral\">\n");
    for ui_file in ui_files {
        xml_content.push_str(&format!("    <file>{}</file>\n", ui_file));
    }
    xml_content.push_str("  </gresource>\n");
    xml_content.push_str("</gresources>\n");
    fs::write(gresource_xml_path, xml_content)
        .expect("No se pudo escribir el archivo gresource.xml");
    glib_build_tools::compile_resources(&["data"], gresource_xml_path, "resources.gresource");
}
