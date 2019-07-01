use distance;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
struct DirData {
    name: String,
    files: HashMap<String, Vec<String>>,
}

impl DirData {
    fn new(dir_name: &str) -> DirData {
        DirData {
            name: String::from(dir_name),
            files: HashMap::new(),
        }
    }

    fn amount_of_files(&self) -> usize {
        self.files.len()
    }
}

pub trait SimilarStrings {
    fn contains_very_similar(&self, _string_to_check: &str) -> bool {
        false
    }
}

impl SimilarStrings for Vec<String> {
    fn contains_very_similar(&self, string_to_check: &str) -> bool {
        for i in self.iter() {
            if distance::levenshtein(i, string_to_check) < 6 {
                return true;
            }
        }
        false
    }
}

fn main() {
    let mut dir_list: Vec<DirData> = Vec::new();

    get_dirs_list(
        Path::new("/home/fort/Programming/rust/diff_between_two_files/build_diff"),
        &mut dir_list,
    )
    .unwrap();

    dir_list.iter_mut().for_each(|mut current_dir| {
        fill_files_list(&mut current_dir);
        match &current_dir.amount_of_files() {
            1 => {
                dbg!(&current_dir.name);
                let mut buff_key = Vec::new();
                for key in current_dir.files.keys() {
                    buff_key.push(key);
                }
                let existing_file_name = Path::new(&buff_key[0]).to_str().unwrap();
                let result_file_name = format!("{}{}", current_dir.name, "/result_diff.txt");
                fs::rename(existing_file_name, result_file_name)
                    .expect("Can't rename existing file");
                println!("Directory has 1 file!");
            }
            2 => {
                dbg!(&current_dir.name);
                create_diff(&current_dir).expect("Can't create diff!");
            }
            _ => {
                // This case can create UB!
                dbg!(&current_dir.name);
                // If diff file already exist, recreate it
                let result_file_name = format!("{}{}", current_dir.name, "/result_diff.txt");
                let result_file = Path::new(&result_file_name);
                if result_file.exists() {
                    fs::remove_file(result_file);
                    println!("Deleted result_diff.txt");
                }

                create_diff(&current_dir).expect("Can't create diff!");
            }
        }

        let result_file_name1 = format!("{}{}", current_dir.name, "/old.txt");
        let result_file1 = Path::new(&result_file_name1);
        let result_file_name2 = format!("{}{}", current_dir.name, "/old — копия.txt");
        let result_file2 = Path::new(&result_file_name2);
        let dir_for_delete = vec![result_file1, result_file2];
        dir_for_delete.iter().for_each(|unnecessary_file| {
            if unnecessary_file.exists() {
                fs::remove_file(unnecessary_file).expect("Can't remove file!");
                println!("File {} was deleted", unnecessary_file.to_str().unwrap());
            }
        })
    });
}

fn create_diff(current_dir: &DirData) -> io::Result<()> {
    // Get all values from map
    let mut buff_values = Vec::new();
    for value in current_dir.files.values() {
        buff_values.push(value);
    }

    write_diff_in_file(
        &current_dir.name,
        &get_files_diff(buff_values[0], buff_values[1]),
    )
    .expect("Fail");
    println!("Successfully created result_diff.txt.");
    Ok(())
}

fn write_diff_in_file(path: &str, diff: &Vec<String>) -> io::Result<()> {
    let buff_path = Path::new(path);
    let result_file_name = buff_path.to_str().unwrap().to_owned() + "/result_diff.txt";
    let mut result_diff_file = File::create(result_file_name).expect("Can not create file!");
    for line in diff.iter() {
        write!(&mut result_diff_file, "{}\n", line);
    }

    Ok(())
}

fn get_files_diff(first_file_data: &Vec<String>, second_file_data: &Vec<String>) -> Vec<String> {
    let mut diff = Vec::new();

    if first_file_data.len() < second_file_data.len() {
        for i in second_file_data.iter() {
            if !first_file_data.contains_very_similar(i) {
                diff.push(i.clone())
            }
        }
    } else {
        for i in first_file_data.iter() {
            if !second_file_data.contains_very_similar(i) {
                diff.push(i.clone())
            }
        }
    }

    dbg!(&diff.len());
    diff
}

// Get list of directories
// TODO: it is a good candidate for create generic func with get_dir_list
fn fill_files_list(root_struct: &mut DirData) -> io::Result<()> {
    for entry in fs::read_dir(&root_struct.name)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.to_str().unwrap().clone();
            root_struct.files.insert(
                file_name.to_string().clone(),
                read_file_data(file_name).unwrap(),
            );
        }
    }

    Ok(())
}

fn read_file_data(file_name: &str) -> io::Result<Vec<String>> {
    let file_desc = File::open(file_name).unwrap();

    let mut strings_in_file = Vec::new();
    for line in BufReader::new(file_desc).lines() {
        strings_in_file.push(line?);
    }
    Ok(strings_in_file)
}

// Get list of directories
fn get_dirs_list(dir: &Path, dir_list: &mut Vec<DirData>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dir_list.push(DirData::new(path.clone().to_str().unwrap()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_dir_data_new() {
        let dir_desc = DirData::new("test");
        let DirData { name, .. } = dir_desc;

        assert_eq!(name, "test");
    }

    #[test]
    fn levenshtein_distance() {
        let old_format = "    npremit_dns.cpp(24): warning C4267: 'initializing' : conversion from 'size_t' to 'unsigned short', possible loss of data";
        let new_format = "    npremit_dns.cpp(24): warning C4267: 'initializing': conversion from 'size_t' to 'unsigned short', possible loss of data";

        assert_eq!(1, distance::levenshtein(old_format, new_format));
    }

    #[test]
    fn contains_very_similar() {
        let mut test_vec: Vec<String> = Vec::new();
        test_vec.push("    npremit_dns.cpp(24): warning C4267: 'initializing' : conversion from 'size_t' to 'unsigned short', possible loss of data".to_string());

        assert_eq!(true, test_vec.contains_very_similar("    npremit_dns.cpp(24): warning C4267: 'initializing': conversion from 'size_t' to 'unsigned short', possible loss of data"));
    }

    #[test]
    fn another_levenshtein_distance() {
        let mut test_vec: Vec<String> = Vec::new();
        let new_format = "gmock-spec-builders.h(864): warning C4251: 'testing::internal::ExpectationBase::untyped_actions_': class 'std::vector>' needs to have dll-interface to be used by clients of class 'testing::internal::ExpectationBase'";

        let old_format = "gmock-spec-builders.h(864): warning C4251: 'testing::internal::ExpectationBase::untyped_actions_' : class 'std::vector<_Ty>' needs to have dll-interface to be used by clients of class 'testing::internal::ExpectationBase'";

        assert_eq!(5, distance::levenshtein(old_format, new_format));
    }
}
