use std::{
    error::Error,
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

const SCHEMA_DIR: &str = "./flatbuffers-schema";
const SCHEMA_DIR_TEMP: &str = "./flatbuffers-schemaTEMP";
const OUT_FILE: &str = "./src/flat_wrapper/planus_flat.rs";

// this is pretty janky, but it works

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=flatbuffers-schema");
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
        .collect::<Vec<_>>()
        .join("");

    let temp_file_paths: Vec<PathBuf> = fbs_file_names
        .iter()
        .map(|x| PathBuf::from(SCHEMA_DIR_TEMP).join(x))
        .collect();

    if !fs::exists(SCHEMA_DIR_TEMP)? {
        fs::create_dir(SCHEMA_DIR_TEMP)?;
    }

    // let required_array_regex = Regex::new(r"([a-z_A-Z]+: *\[[A-Z-a-z]+\]) *\(required\)")?;

    for (fbs_file_path, temp_file_path) in fbs_file_paths.iter().zip(temp_file_paths.iter()) {
        let mut contents = fs::read_to_string(fbs_file_path)?;

        // planus doesn't support multiple root_types
        // removing them doesn't seem to do much
        contents = contents.replace("root_type", "// root_type");

        // comment all existing includes
        contents = contents.replace("include \"", "// include \"");

        // include all files (planus isn't smart enough to auto include the parent file)
        contents = include_all_str.clone() + &contents;

        // Don't do this, it breaks the binary rep of the schema
        // // planus can't generate default impls for structs that have
        // // arrays with (required) fields.
        // // we replace (required) with a default value
        // contents = required_array_regex
        //     .replace_all(&contents, "$1=[]")
        //     .to_string();

        fs::File::create(temp_file_path)?.write_all(contents.as_bytes())?;
    }

    let start_time_planus = Instant::now();

    let declarations =
        planus_translation::translate_files(&temp_file_paths).ok_or("planus translation failed")?;
    let mut res = planus_codegen::generate_rust(&declarations)?;

    // No idea why planus renames RLBot to RlBot but this fixes it
    res = res.replace("RlBot", "RLBot");

    let time_taken = format!(
        "// build.rs took {:?} of which planus took {:?}\n",
        Instant::now().duration_since(start_time),
        Instant::now().duration_since(start_time_planus)
    );

    fs::File::create(OUT_FILE)?.write_all(&[time_taken.as_bytes(), res.as_bytes()].concat())?;

    fs::remove_dir_all(SCHEMA_DIR_TEMP)?;

    Ok(())
}
