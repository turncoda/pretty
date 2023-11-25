use clap::Parser;
use ordered_float::OrderedFloat;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use unreal_asset::exports::Export;
use unreal_asset::exports::ExportBaseTrait;
use unreal_asset::properties::Property;
use unreal_asset::types::PackageIndex;
use unreal_asset::Asset;

/// Parse Pseudoregalia time trial text files into cooked Unreal Engine data tables
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to time trial text file
    #[arg(short, long)]
    input: String,

    /// Path to write uasset file
    #[arg(short, long)]
    output: String,
}

#[derive(Debug)]
struct Waypoint {
    x: f64,
    y: f64,
    z: f64,
    gates: Vec<i32>,
    name: String,
    number: usize,
}

struct ParseError {
    line_number: usize,
    line: String,
    token_number: usize,
    token: String,
    error: String,
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {}: '{}', token {}: '{}'\nReason: {}",
            self.line_number, self.line, self.token_number, self.token, self.error
        )
    }
}

#[derive(Debug, Default)]
struct WaypointIndexMap {
    hash_map: HashMap<String, usize>,
    number_map: HashMap<usize, usize>,
    vec: Vec<Waypoint>,
}

impl WaypointIndexMap {
    fn new() -> WaypointIndexMap {
        WaypointIndexMap::default()
    }

    fn get_by_name(&self, name: &str) -> Option<usize> {
        self.hash_map.get(name).cloned()
    }

    fn get_by_number(&self, number: &usize) -> Option<usize> {
        self.number_map.get(number).cloned()
    }

    fn add(&mut self, waypoint: Waypoint) {
        let name = waypoint.name.clone();
        let number = waypoint.number;
        self.vec.push(waypoint);
        self.hash_map.insert(name, self.vec.len() - 1);
        self.number_map.insert(number, self.vec.len() - 1);
    }

    fn len(&self) -> usize {
        self.vec.len()
    }
}

const DT_WAYPOINT_UASSET: &[u8] = include_bytes!("DT_SampleWaypointTable.uasset");
const DT_WAYPOINT_UEXP: &[u8] = include_bytes!("DT_SampleWaypointTable.uexp");

fn main() -> Result<(), ParseError> {
    let args = Args::parse();

    let tt_data = std::fs::read_to_string(args.input).unwrap();
    let waypoint_map = parse_time_trial_data(&tt_data)?;

    let mut asset = Asset::new(
        Cursor::new(DT_WAYPOINT_UASSET),
        Some(Cursor::new(DT_WAYPOINT_UEXP)),
        unreal_asset::engine_version::EngineVersion::VER_UE5_1,
        None,
    )
    .unwrap();

    let mut fnames = vec![];
    for waypoint in waypoint_map.vec.iter() {
        fnames.push(asset.add_fname(&waypoint.name));
    }
    let new_main_export_name = Path::new(&args.output)
        .file_stem()
        .unwrap()
        .to_string_lossy();
    let new_main_export_fname = asset.add_fname(&new_main_export_name);

    let export = asset.get_export_mut(PackageIndex::new(1)).unwrap();
    assert!(export.get_base_export().object_name.get_owned_content() == "DT_SampleWaypointTable");
    export.get_base_export_mut().object_name = new_main_export_fname;

    if let Export::DataTableExport(export) = export {
        let row = &export.table.data[1];
        let mut new_rows = vec![];
        for (waypoint, fname) in waypoint_map.vec.iter().zip(fnames.iter()) {
            let mut new_row = row.clone();
            for field in &mut new_row.value {
                if let Property::DoubleProperty(prop) = field {
                    if prop.name.get_owned_content().starts_with("x") {
                        prop.value = OrderedFloat(waypoint.x);
                    }
                    if prop.name.get_owned_content().starts_with("y") {
                        prop.value = OrderedFloat(waypoint.y);
                    }
                    if prop.name.get_owned_content().starts_with("z") {
                        prop.value = OrderedFloat(waypoint.z);
                    }
                }
                if let Property::ArrayProperty(prop) = field {
                    if let Property::IntProperty(element) = prop.value[0].clone() {
                        prop.value.clear();
                        for gate in &waypoint.gates {
                            let mut clone = element.clone();
                            clone.value = *gate;
                            prop.value.push(Property::from(clone));
                        }
                    }
                }
                if let Property::NameProperty(prop) = field {
                    prop.value = fname.clone();
                }
            }
            new_row.name = fname.clone();
            new_rows.push(new_row);
        }
        export.table.data = new_rows;
    }

    let output_uasset_path = Path::new(&args.output);
    let mut output_uasset_file = File::create(output_uasset_path).unwrap();
    let output_uexp_path = output_uasset_path.with_extension("uexp");
    let mut output_uexp_file = File::create(output_uexp_path).unwrap();
    asset
        .write_data(&mut output_uasset_file, Some(&mut output_uexp_file))
        .unwrap();

    Ok(())
}

fn parse_time_trial_data(data: &str) -> Result<WaypointIndexMap, ParseError> {
    let mut waypoint_map = WaypointIndexMap::new();
    let mut line_number: usize = 0;
    for line in data.lines() {
        line_number += 1;
        if line.is_empty() {
            continue;
        }
        let mut token_number = 0;
        let mut gates = vec![];
        if waypoint_map.len() > 0 {
            gates.push(waypoint_map.len() as i32 - 1);
        }
        let mut name = format!("Waypoint{}", line_number);
        let tokens: Vec<_> = line.split_whitespace().collect();
        let line = line.to_string();
        if tokens.len() < 3 {
            let error = "fewer than 3 tokens".to_string();
            return Err(ParseError {
                line_number,
                line,
                token_number,
                token: "".to_string(),
                error,
            });
        }
        let x = match i32::from_str_radix(tokens[token_number], 10) {
            Ok(number) => number as f64,
            Err(error) => {
                return Err(ParseError {
                    line_number,
                    line,
                    token_number,
                    token: tokens[token_number].to_string(),
                    error: error.to_string(),
                })
            }
        };
        token_number += 1;
        let y = match i32::from_str_radix(tokens[token_number], 10) {
            Ok(number) => number as f64,
            Err(error) => {
                return Err(ParseError {
                    line_number,
                    line,
                    token_number,
                    token: tokens[token_number].to_string(),
                    error: error.to_string(),
                })
            }
        };
        token_number += 1;
        let z = match i32::from_str_radix(tokens[token_number], 10) {
            Ok(number) => number as f64,
            Err(error) => {
                return Err(ParseError {
                    line_number,
                    line,
                    token_number,
                    token: tokens[token_number].to_string(),
                    error: error.to_string(),
                })
            }
        };
        token_number += 1;
        if token_number >= tokens.len() {
            waypoint_map.add(Waypoint {
                x,
                y,
                z,
                gates,
                name,
                number: line_number,
            });
            continue;
        }
        if tokens[token_number] == "<" {
            // name field omitted
            token_number += 1;
        } else {
            // consume name field
            if let Err(error) = validate_name_token(tokens[token_number]) {
                return Err(ParseError {
                    line_number,
                    line,
                    token_number,
                    token: tokens[token_number].to_string(),
                    error,
                });
            }
            name = tokens[token_number].to_string();
            token_number += 1;
            if token_number >= tokens.len() {
                waypoint_map.add(Waypoint {
                    x,
                    y,
                    z,
                    gates,
                    name,
                    number: line_number,
                });
                continue;
            }
            if tokens[token_number] != "<" {
                return Err(ParseError {
                    line_number,
                    line,
                    token_number,
                    token: tokens[token_number].to_string(),
                    error: format!("token after name must be gate delimiter '<'"),
                });
            }
            token_number += 1;
        }
        // start parsing gates
        let remaining_tokens = &tokens[token_number..];
        if remaining_tokens.len() > 0 {
            gates.clear();
        }
        for gate_name in remaining_tokens {
            if let Ok(i) = i32::from_str_radix(gate_name, 10) {
                match waypoint_map.get_by_number(&(i as usize)) {
                    Some(i) => gates.push(i as i32),
                    None => {
                        return Err(ParseError {
                            line_number,
                            line,
                            token_number,
                            token: tokens[token_number].to_string(),
                            error: format!("unrecognized waypoint number"),
                        })
                    }
                };
                continue;
            }
            match waypoint_map.get_by_name(gate_name) {
                Some(i) => gates.push(i as i32),
                None => {
                    return Err(ParseError {
                        line_number,
                        line,
                        token_number,
                        token: tokens[token_number].to_string(),
                        error: format!("unrecognized waypoint name"),
                    })
                }
            };
        }

        waypoint_map.add(Waypoint {
            x,
            y,
            z,
            gates,
            name,
            number: line_number,
        });
    }
    Ok(waypoint_map)
}

fn validate_name_token(token: &str) -> Result<(), String> {
    let re = Regex::new(r"^[_A-Za-z][-_A-Za-z0-9]*$").unwrap();
    if re.is_match(token) {
        Ok(())
    } else {
        Err(format!("Name token '{}' does not match regex.", token))
    }
}
