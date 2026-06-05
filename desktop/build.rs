use std::{fs, path::Path, process::Command};

const RE_RUN_DIRS: &[&str] = &["src", "po"];

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

fn collect_files_by_ext(dir: &Path, ext: &str, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_by_ext(&path, ext, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                if let Some(p) = path.to_str() {
                    files.push(p.to_string());
                }
            }
        }
    }
}

fn generate_pot_file(ui_out: &Path) {
    let po_dir = Path::new("po");
    if !po_dir.exists() {
        let _ = fs::create_dir_all(po_dir);
    }

    let mut files_to_scan = Vec::new();
    collect_files_by_ext(Path::new("src"), "rs", &mut files_to_scan);
    collect_files_by_ext(ui_out, "ui", &mut files_to_scan);

    if files_to_scan.is_empty() {
        return;
    }

    let mut cmd = Command::new("xgettext");
    cmd.args([
        "--from-code=UTF-8",
        "-o",
        "po/umbral.pot",
        "--omit-header",
        "--no-location",
        "--no-wrap",
    ]);
    cmd.args(&files_to_scan);
    let _ = cmd.status();
}

fn compile_mo_files() {
    let po_dir = Path::new("po");
    if let Ok(entries) = fs::read_dir(po_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("po") {
                let lang = path.file_stem().unwrap().to_str().unwrap();
                let mo_dir_path = format!("locale/{}/LC_MESSAGES", lang);
                fs::create_dir_all(&mo_dir_path).unwrap();

                let _ = Command::new("msgfmt")
                    .args([
                        "-o",
                        &format!("{}/umbral.mo", mo_dir_path),
                        path.to_str().unwrap(),
                    ])
                    .status();
            }
        }
    }
}

fn main() {
    for dir in RE_RUN_DIRS {
        println!("cargo:rerun-if-changed={}", dir);
    }
    let ui_src = Path::new("src/ui");
    let ui_out = Path::new("data/ui");
    let gresource_xml_path = "data/resources.gresource.xml";
    fs::create_dir_all(ui_out).unwrap();
    let mut xml_content = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gresources>\n  <gresource prefix=\"/edu/unesum/umbral\">\n",
    );
    if let Ok(entries) = fs::read_dir(ui_src) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("blp") {
                let file_name = path.file_stem().unwrap().to_str().unwrap();
                let output = ui_out.join(format!("{}.ui", file_name));

                compile_blueprint(&path, &output);
                xml_content.push_str(&format!("    <file>ui/{}.ui</file>\n", file_name));
            }
        }
    }
    xml_content.push_str("  </gresource>\n</gresources>\n");
    fs::write(gresource_xml_path, xml_content).expect("No se pudo escribir gresource.xml");
    glib_build_tools::compile_resources(&["data"], gresource_xml_path, "resources.gresource");
    generate_pot_file(ui_out);
    compile_mo_files();
}
