use super::Version;
use serde::{Deserialize, Serialize};

// TODO: give better name
#[derive(Deserialize, Serialize)]
pub struct Entry {
    pub name: String,
    pub version: Version,
}

impl Entry {
    pub fn new(name: String, version: Version) -> Self {
        Self { name, version }
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct Manifest {
    pub packages: Vec<Entry>,
}

impl Manifest {
    pub fn get(&self, name: &str) -> Option<&Entry> {
        self.packages.iter().find(|entry| entry.name == name)
    }

    pub fn installed_count(&self) -> usize {
        self.packages.len()
    }

    pub fn installed(&self, pkg_name: &str) -> bool {
        self.packages.iter().any(|pkg| pkg.name == pkg_name)
    }

    /// Removes a package from the manifest.
    pub fn remove(&mut self, pkg_name: &str) {
        self.packages.retain(|entry| entry.name != pkg_name);
    }

    /// Updates or inserts a package.
    pub fn upsert(&mut self, entry: Entry) {
        let mut index: Option<usize> = None;
        for (i, item) in self.packages.iter().enumerate() {
            if item.name == entry.name {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(index) => {
                let _ = std::mem::replace(&mut self.packages[index], entry);
            }
            None => self.packages.push(entry),
        }
    }
}
