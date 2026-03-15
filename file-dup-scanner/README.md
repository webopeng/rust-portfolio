## Project: Asynchronous File Duplicate Scanner

# dupe-scan  

[  
[  
[  
[  

**dupe-scan** is a high-performance Windows command-line utility written in **Rust** that finds duplicate files across all connected drives. It uses the **BLAKE3** hashing algorithm for speed and accuracy, scanning drives in parallel to efficiently identify redundant large files.  

***

## ✨ Features

- 🔍 **Automatic drive detection** — scans all active Windows drives (A:–Z:).  
- ⚡ **Multi-threaded** — powered by [Rayon](https://crates.io/crates/rayon) for fast parallel hashing.  
- 🧾 **Configurable size thresholds** — skip small files (default: 11 MB).  
- 💾 **Duplicate grouping** — detects and groups files with identical BLAKE3 hashes.  
- 🧰 **PowerShell cleanup script** — generates `delete_dupes.ps1` to safely remove redundant files.  
- 🧮 **Optional limit mode** — set a maximum number of files to process for testing.  

***

## 🛠️ Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)
- Windows OS (uses drive letters and PowerShell script output)

### Build from source
```bash
git clone https://github.com/yourusername/dupe-scan.git
cd dupe-scan
cargo build --release
```

The compiled executable will be available at:
```
target/release/dupe-scan.exe
```

You can copy this to a folder in your system `PATH` to run it from anywhere.

***

## 🚀 Usage

### Basic command
```bash
dupe-scan
```

### With options
```bash
dupe-scan --min-size 20MB --limit 1000
```

### Arguments
| Flag | Description | Default |
|------|--------------|----------|
| `--min-size` | Minimum file size to scan (supports MB or GB suffix). | `11MB` |
| `--limit` | Maximum number of files to process for testing. | None |
| `--json` | (Reserved) Output results in JSON format. | False |

***

## 📄 Example Output

After running `dupe-scan`, two files will be generated in the working directory:  

**`dupe_list.txt`**
```
KEEP: C:\Videos\lecture1.mp4
DUP : D:\Backup\lecture1_copy.mp4
DUP : E:\External\old\lecture1.mp4

KEEP: D:\Photos\vacation.jpg
DUP : E:\Archive\vacation_copy.jpg
```

**`delete_dupes.ps1`**
```powershell
Remove-Item -LiteralPath "D:\Backup\lecture1_copy.mp4" -Force
Remove-Item -LiteralPath "E:\External\old\lecture1.mp4" -Force
Remove-Item -LiteralPath "E:\Archive\vacation_copy.jpg" -Force
```

Then you can manually review `dupe_list.txt` or run the PowerShell script to clean up duplicates.

***

## ⚙️ How It Works

1. **Drive detection:** Finds all mounted drives on Windows (`A:` through `Z:`).  
2. **File filtering:** Recursively scans for files above `--min-size`.  
3. **Parallel hashing:** Computes BLAKE3 hashes in 4 MB chunks using all available CPU cores.  
4. **Grouping:** Organizes files with matching hashes into duplicate sets.  
5. **Output:** Reports duplicates and generates safe delete scripts for cleanup.

***

## 📊 Performance Benchmark
The following benchmarks are example numbers; replace them with your own measurements from your hardware and dataset.

## Test setup:

CPU: 8-core / 16-thread

Disk: NVMe SSD

Dataset: ~100 GB of mixed media files, --min-size 50MB

| Mode | Threads | Files scanned | Total size | Time | Speedup |
| Single-threaded | 1 |	8,000 | 100 GB | 12 min | 1.0× |
| Parallel (Rayon) | 8 | 8,000 | 100 GB | 3.5 min | 3.4× |
To reproduce something similar:

bash
# Single-threaded (example: by disabling Rayon or using a debug build)
dupe-scan --min-size 50MB

# Parallel (release build with Rayon)
dupe-scan --min-size 50MB
Update the table with your real timings from time dupe-scan ... or PowerShell’s Measure-Command.

***

## 📦 Example Workflow

```bash
> dupe-scan --min-size 50MB
Scanning drives: ["C:\\", "D:\\", "E:\\"]
Limiting to first 5000 files for testing
Total duplicate size: 8,962,457,600 bytes
```

Output:
```
✅ dupe_list.txt generated
✅ delete_dupes.ps1 generated
```

***

## 🪪 License

Licensed under the [MIT License](LICENSE).  
Developed with ❤️ in Rust.

***

Would you like me to add a **project logo/banner** and “Performance Benchmark” section comparing single-thread vs. parallel runs? That would make the README even more polished and professional-looking.