use crate::error::Error;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::io::Read;
use regex::Regex;
use std::collections::HashMap;

/// Represents a database migration
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i32,
    pub description: String,
    pub sql: String,
    pub path: PathBuf,
}

/// Manager for handling database migrations
#[derive(Debug, Clone, Default)]
pub struct Migrations {
    migrations: HashMap<i32, Migration>,
}

impl Migrations {
    /// Create a new empty migrations manager
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }
    
    /// Add a migration to the collection
    pub fn add(&mut self, migration: Migration) {
        self.migrations.insert(migration.version, migration);
    }
    
    /// Get all migrations in ordered sequence
    pub fn get_all(&self) -> Vec<Migration> {
        let mut versions: Vec<i32> = self.migrations.keys().cloned().collect();
        versions.sort();
        
        versions.into_iter()
            .filter_map(|v| self.migrations.get(&v).cloned())
            .collect()
    }
    
    /// Load migrations from a directory
    pub fn load_from_directory(&mut self, dir_path: &Path) -> Result<(), Error> {
        tracing::info!("Loading migrations from {}", dir_path.display());
        
        if !dir_path.exists() {
            return Err(Error::Unknown(format!("Migrations directory does not exist: {}", dir_path.display())));
        }
        
        let version_regex = Regex::new(r"^V(\d+)__(.+)$").unwrap();
        
        // First find all version directories
        let version_dirs = fs::read_dir(dir_path)
            .map_err(|e| Error::Unknown(format!("Failed to read migrations directory: {}", e)))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if let Some(captures) = version_regex.captures(&dir_name) {
                    let version = captures.get(1).unwrap().as_str().parse::<i32>().unwrap();
                    let description = captures.get(2).unwrap().as_str().replace('_', " ");
                    Some((version, description, entry))
                } else {
                    None
                }
            })
            .collect::<Vec<(i32, String, DirEntry)>>();
        
        for (version, description, dir_entry) in version_dirs {
            // Find all SQL files in this version directory
            let mut sql_files = Vec::new();
            for entry in fs::read_dir(dir_entry.path())
                .map_err(|e| Error::Unknown(format!("Failed to read migration directory: {}", e)))?
                .filter_map(Result::ok)
                .filter(|e| {
                    e.path().is_file() && 
                    e.path().extension().map_or(false, |ext| ext == "sql")
                })
            {
                sql_files.push(entry);
            }
            
            // Sort SQL files by name to ensure they're executed in the correct order
            sql_files.sort_by_key(|f| f.file_name().to_string_lossy().to_string());
            
            // Combine all SQL content
            let mut combined_sql = String::new();
            for file in sql_files {
                let mut file_content = String::new();
                let path = file.path();
                let mut file_handle = fs::File::open(&path)
                    .map_err(|e| Error::Unknown(format!("Failed to open migration file {}: {}", path.display(), e)))?;
                
                file_handle.read_to_string(&mut file_content)
                    .map_err(|e| Error::Unknown(format!("Failed to read migration file {}: {}", path.display(), e)))?;
                
                // Add file content to combined SQL with a separator comment
                combined_sql.push_str(&format!(
                    "\n-- From file: {} \n{}\n",
                    file.file_name().to_string_lossy(),
                    file_content
                ));
            }
            
            // Add the migration (clone description before it's moved)
            let description_clone = description.clone();
            self.add(Migration {
                version,
                description: description_clone.clone(), // Use the clone here too
                sql: combined_sql,
                path: dir_entry.path(),
            });
            
            tracing::info!("Loaded migration V{}: {}", version, description_clone);
        }
        
        tracing::info!("Loaded {} migrations", self.migrations.len());
        
        Ok(())
    }
}
