# macos-fseventsd

A simple macOS File System Events Disk Log Stream (FsEventsd) parser (and library) written in Rust!  
FsEvent records on macOS keeps track of file changes on a system.  This simple library lets you parse these records.  
The example binary can parse these records to a csv and json file.  

The example binary can be run on a live system or you can provide a directory containing FsEvent files.
# How to use
1. Download `fsevents_parser` binary (or compile and build example yourself)  
   a. You can compile the example binary by running `cargo build --release --example fseventsd_parser`  
3. If running on a live system, run `sudo ./fsevents_parser`  
   a. You need root access to read FsEvent records on a live system
3. If FsEvents have been acquired via another tool, run `./fsevents_parser <path to directory containing FsEvent files>`
4. `fsevents_parser` will output a CSV file and a json.

# Use Case
Parsing FsEvents is mainly useful for forensic investigations. You can parse FsEvents to determine if a file previously existed on disk.  
Ex: Check if malware existed on a system or if a user downloaded a malicious file from the Internet or opened a phishing document.

# FsEvents Data
FsEvent records on macOS keeps track of of file changes on a system.  In addition, FsEvents records can be created on additional drives that are formatted with APFS (or HFS+).  
A FsEvent files can exist in two locations depending on the macOS version:
1. `/.fsevents` macOS versions below BigSur
2. `/System/Volumes/Data/.fseventsd/` macOS BigSur and higher

You need `root` permissions in order to read the files.  
FsEvent files are compressed with Gzip and are stored in a binary format that must be parsed.  
Data that can be extracted from FsEvent data includes:
1. Path for file record
2. File change event (Event Flags). Such as Created, Removed, Changed, etc.
3. Event ID
4. Node ID

FsEvents can be disabled for a volume by creating a file named `no_log` in the root directory.

# References
https://github.com/libyal/dtformats/blob/main/documentation/MacOS%20File%20System%20Events%20Disk%20Log%20Stream%20format.asciidoc  
https://www.crowdstrike.com/blog/using-os-x-fsevents-discover-deleted-malicious-artifact/  
https://eclecticlight.co/2017/09/12/watching-macos-file-systems-fsevents-and-volume-journals/  

# Other FsEvent parsing tools
https://github.com/dlcowen/FSEventsParser
