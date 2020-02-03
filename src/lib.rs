use glob::glob;
use std::fs::File;
use std::io::{BufReader, BufRead, Read, Write};
use std::path::PathBuf;
use regex::Regex;

pub fn deserialize_entity(component_dir: &str, output_path: &str) {
    let mut header = "use specs::{World, WorldExt, Builder, EntityBuilder};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use regex::Regex;
".to_string();

    let mut pre_content = "
pub fn create_from_yml(world: &mut World, yml_dir: &str) {
    let paths = fs::read_dir(yml_dir).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let file = File::open(path.as_path()).unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text);

        let entities: Vec<&str> = text.split(\"\\n---\\n\").collect();

        for entity_str in entities {
            create_entity(world, entity_str);
        }
    }
}

fn create_entity(world: &mut World, entity_str: &str) {
    let mut entity_builder = world.create_entity();
    let re = Regex::new(r\"\\n*([^\\s]+):\\n((\\s+[^\\n]+(\\n|$))+)\").unwrap();
    for caps in re.captures_iter(entity_str) {
        let component_name = caps.get(1).unwrap().as_str();
        let component_str = caps.get(2).unwrap().as_str();
        entity_builder = append_component(entity_builder, component_name, component_str);
    }
    entity_builder.build();
}

fn append_component<'a>(entity_builder: EntityBuilder<'a>, name: &str, serialized: &str) -> EntityBuilder<'a> {
    match name {".to_string();

    let mut suf_content = "
        _ => entity_builder,
    }
}".to_string();

    let mut import_component = String::new();
    let mut kk = String::new();

    let component_infos = get_component_infos(component_dir);
    for info in component_infos {
        if info.components.len() > 0 {
            import_component += format!(
                "use crate::components::{}::{{{}}};\n", info.file_name, info.components.join(", ")
            ).as_str();
        }
        for component in info.components {
            kk += format!(
                "
        \"{0}\" => {{
            let component = serde_yaml::from_str::<{0}>(serialized).unwrap();
            entity_builder.with(component)
        }},", component).as_str();
        }
    }

    let mut file = File::create((output_path.to_string()).as_str()).unwrap();
    file.write_all(&header.into_bytes());
    file.write_all(&import_component.into_bytes());
    file.write_all(&pre_content.into_bytes());
    file.write_all(&kk.into_bytes());
    file.write_all(&suf_content.into_bytes());
}

#[derive(Debug)]
pub struct ComponentInfo {
    file_path: PathBuf,
    file_name: String,
    components: Vec<String>,
}

pub fn get_component_infos(component_dir: &str) -> Vec<ComponentInfo> {
    let mut result = Vec::new();

    let re = Regex::new(r"#\[derive\([^\)]*Component[^\)]*Deserialize[^\)]*\)\][^;]*?struct\s+([^\s]+)\s*\{").unwrap();

    let mut path = component_dir.to_string();
    path.push_str("/*.rs");
    for entry in glob(path.as_str()).expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            let f = File::open(path.clone()).expect("Failed to open file");
            let mut reader = BufReader::new(f);
            let mut text = String::new();
            reader.read_to_string(&mut text);

            let mut item = ComponentInfo {
                file_path: path.clone(),
                file_name: path.file_stem().unwrap().to_str().unwrap().to_string(),
                components: vec![],
            };
            for caps in re.captures_iter(&text) {
                let component_name = caps.get(1).unwrap().as_str().to_string();
                item.components.push(component_name);
            }
            result.push(item);
        }
    }
    result
}