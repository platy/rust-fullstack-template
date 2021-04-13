use std::{
    env,
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let web_project_dir: &Path = "web".as_ref();

    let (web_sources, web_artifacts) = build_web(web_project_dir)?;

    for path in web_sources {
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    fs::create_dir_all(&out_dir)?;
    let inclusion_file_path = Path::new(&out_dir).join("web.rs");
    println!(
        "Generating inclusion file at {:?} ...",
        &inclusion_file_path
    );
    let mut generated_web_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(inclusion_file_path)?;
    // TODO use template for default html
    // writeln!(
    //     generated_web_file,
    //     r#"pub const INDEX_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/{}"));"#,
    //     web_project_dir.join("index.html").to_str().unwrap(),
    // )?;
    writeln!(generated_web_file)?;
    let mut artifact_map_insertions = String::new();
    for artifact in web_artifacts {
        let artifact_name = artifact.file_name().unwrap().to_str().unwrap();
        let identifier = artifact_name
            .replace('/', "_")
            .replace('.', "_")
            .replace('-', "_")
            .to_uppercase();
        let artifact = artifact.to_str().unwrap();
        writeln!(
            generated_web_file,
            r#"pub const {}: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/{}"));"#,
            identifier, artifact,
        )?;
        artifact_map_insertions.push_str(&format!(
            "        \"{}\" => Some({}),\n",
            artifact_name, identifier,
        ));
    }
    writeln!(generated_web_file,)?;
    generated_web_file.write_all(
        b"pub fn artifact(path: &str) -> Option<&'static [u8]> {
    match path {
",
    )?;
    generated_web_file.write_all(artifact_map_insertions.as_bytes())?;
    generated_web_file.write_all(
        b"
        _ => None,
    }
}
",
    )?;

    println!("build.rs completion");

    Ok(())
}

fn build_web(web_crate_path: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Box<dyn Error>> {
    let web_out_path = web_crate_path.join("pkg");
    let build_opts = wasm_pack::command::build::BuildOptions {
        path: Some(web_crate_path.to_path_buf()),
        scope: None,
        mode: wasm_pack::install::InstallMode::Normal,
        disable_dts: false,
        target: wasm_pack::command::build::Target::Web,
        debug: true,
        dev: true,
        release: false,
        profiling: false,
        out_dir: "pkg".to_owned(),
        out_name: Some("app".to_owned()),
        extra_options: vec![],
    };
    wasm_pack::command::build::Build::try_from_opts(build_opts)?.run()?;

    let artifacts: Vec<PathBuf> = fs::read_dir(&web_out_path)?
        .map(|dir_entry| dir_entry.unwrap().path())
        .collect();

    let sources: Vec<PathBuf> = [
        "index.html",
        "Cargo.toml",
        "Cargo.lock",
        "src/lib.rs",
        "../shared",
    ] // TODO detect these files, dir doesn't work well, include shared
    .iter()
    .copied()
    .map(|file_name| web_crate_path.join(file_name))
    .collect();

    Ok((sources, artifacts))
}
