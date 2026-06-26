use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use fs2::FileExt; // Brings trait for lock_exclusive into scope
use crate::hf_ingest::UnifiedLedger;

pub struct LedgerStorageManager {
    file_path: String,
}

impl LedgerStorageManager {
    pub fn new(path: &str) -> Self {
        Self {
            file_path: path.to_string(),
        }
    }

    /// Backwards compatibility wrapper that routes to the secure transactional commit
    pub fn commit_ledger(&self, ledger: &UnifiedLedger) -> Result<(), Box<dyn std::error::Error>> {
        self.transactional_commit(ledger)
    }

    /// Safely updates the ledger on disk using an atomic swap strategy.
    /// It locks the target file, writes to a shadow temp file, and swaps them to prevent corruption.
    pub fn transactional_commit(&self, runtime_ledger: &UnifiedLedger) -> Result<(), Box<dyn std::error::Error>> {
        let target_path = Path::new(&self.file_path);
        
        // 1. Open or create the primary lockfile to block other processes (like Python)
        let lock_path = target_path.with_extension("lock");
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)?;
            
        // Block thread until an exclusive OS lock is acquired
        lock_file.lock_exclusive()?;

        // 2. Write the new JSON state to a temporary sibling file (.json.tmp)
        let temp_path = target_path.with_extension("tmp");
        let mut temp_file = File::create(&temp_path)?;
        
        let serialized_data = serde_json::to_string_pretty(runtime_ledger)?;
        temp_file.write_all(serialized_data.as_bytes())?;
        
        // Force OS kernel memory page cache to flush cleanly to physical silicon
        temp_file.sync_all()?;

        // 3. Atomically rename the temp file to the target file.
        // On POSIX systems, this operation is atomic at the VFS layer.
        std::fs::rename(&temp_path, target_path)?;

        // 4. Release the lock explicitly
        lock_file.unlock()?;
        
        Ok(())
    }

    /// Read-Locking safe loader to ingest the disk state back into the engine
    pub fn transactional_load(&self) -> Result<UnifiedLedger, Box<dyn std::error::Error>> {
        let target_path = Path::new(&self.file_path);
        if !target_path.exists() {
            return Err("Ledger file does not exist on disk yet".into());
        }

        let lock_path = target_path.with_extension("lock");
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(lock_path)?;
        
        // Acquire shared read lock (multiple readers can read, but writers will block)
        lock_file.lock_shared()?;

        let mut file = File::open(target_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        lock_file.unlock()?;

        let ledger: UnifiedLedger = serde_json::from_str(&contents)?;
        Ok(ledger)
    }
}