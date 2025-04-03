use std::{
    error::Error,
    fs::{self},
    io::Write,
    path::Path,
    time::Instant,
};

const SCHEMA_DIR: &str = "../../flatbuffers-schema";
const OUT_FILE: &str = "./src/planus_flat.rs";

// this is pretty janky, but it works

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=../../flatbuffers-schema");
    println!("cargo:rerun-if-changed=build.rs");

    let start_time = Instant::now();

    if !Path::new(SCHEMA_DIR).exists() {
        Err("Couldn't find flatbuffers schema folder")?;
    }

    let fbs_file_paths: Vec<_> = fs::read_dir(SCHEMA_DIR)?
        .map(|x| x.unwrap().path())
        .filter(|x| x.is_file() && x.extension().map(|x| x.to_str()) == Some(Some("fbs")))
        .collect();

    let fbs_file_names: Vec<_> = fbs_file_paths
        .iter()
        .map(|x| x.file_name().unwrap().to_str().unwrap().to_owned())
        .collect();

    let include_all_str = fbs_file_names
        .iter()
        .map(|x| format!("include \"{x}\";"))
        .collect::<String>();

    let files = fbs_file_paths
        .iter()
        .map(|fbs_file_path| {
            let mut contents = fs::read_to_string(fbs_file_path).expect("failed to read file");

            // planus doesn't support multiple root_types
            // removing them doesn't seem to do much
            contents = contents.replace("root_type", "// root_type");

            // comment all existing includes
            contents = contents.replace("include \"", "// include \"");

            // include all files (since we're removing root_types the root_types aren't auto-included)
            contents = include_all_str.clone() + &contents;

            (
                fbs_file_path
                    .strip_prefix(SCHEMA_DIR)
                    .expect("failed to strip SCHEMA_DIR prefix from file path"),
                contents,
            )
        })
        .collect::<Vec<_>>();

    let start_time_planus = Instant::now();

    let declarations =
        planus_translation::translate_files_from_memory_with_options(&files, Default::default());
    let mut res = planus_codegen::generate_rust(&declarations)?;

    // No idea why planus renames RLBot to RlBot but this fixes it
    res = res.replace("RlBot", "RLBot");

    // flatbuffers-schemaTEMP looks ugly, fix it
    res = res.replace("flatbuffers-schemaTEMP", "rlbot/flatbuffers-schema");

    let now = Instant::now();
    let time_taken = format!(
        "// build.rs took {:?} of which planus took {:?}\n",
        now.duration_since(start_time),
        now.duration_since(start_time_planus)
    );

    fs::File::create(OUT_FILE)?.write_all(&[time_taken.as_bytes(), res.as_bytes()].concat())?;

    Ok(())
}
