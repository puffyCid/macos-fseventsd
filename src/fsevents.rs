//! Parse macOS FsEvent data
//!
//! Provides a library to decompress and parse FsEvent files.

use nom::{
    bytes::streaming::take,
    number::streaming::{le_u32, le_u64},
};
use serde::Serialize;
use std::{mem::size_of, str::from_utf8};

#[derive(Debug, Serialize)]
pub struct FsEvents {
    pub flags: String, // Flags associatd with FsEvent record
    pub path: String,  // File path for FsEvent record
    pub node: u64,     // Node ID for FsEvent record
    pub event_id: u64, // Event ID for for FsEvent record
}

#[derive(Debug)]
struct FsEventsHeader {
    signature: u32,   // File signature DLS1 or DLS2
    padding: u32,     // Unknown, possibly padding
    stream_size: u32, // Size of stream of FsEvent records, includes header size
}

impl FsEvents {
    const DISKLOGGERV2: u32 = 0x444c5332;
    const DISKLOGGERV1: u32 = 0x444c5331;

    /// Parse provided FsEvent data
    pub fn fsevents_data(data: &[u8]) -> nom::IResult<&[u8], Vec<FsEvents>> {
        let mut total_fsevents: Vec<FsEvents> = Vec::new();
        let mut input = data;

        // Loop through all the FsEvent data
        // Parse header to get FsEvent stream size
        // Parse FsEvent stream data
        loop {
            let (fsevents_data, fsevents_header) = FsEvents::fsevents_header(input)?;
            if fsevents_header.signature != FsEvents::DISKLOGGERV1
                && fsevents_header.signature != FsEvents::DISKLOGGERV2
            {
                break;
            }

            let header_size = 12;
            let (stream_input, fsevent_data) =
                take(fsevents_header.stream_size - header_size)(fsevents_data)?;

            let (_result, mut fsevents) =
                FsEvents::get_fsevent(fsevent_data, fsevents_header.signature)?;
            total_fsevents.append(&mut fsevents);
            input = stream_input;
            if input.len() == 0 {
                break;
            }
        }

        Ok((input, total_fsevents))
    }

    /// Begin parsing FsEvent stream
    fn get_fsevent(data: &[u8], sig: u32) -> nom::IResult<&[u8], Vec<FsEvents>> {
        let mut input_results = data;
        let mut fsevents_array: Vec<FsEvents> = Vec::new();

        // Parse FsEvent stream and get each FsEvent record
        loop {
            let (input_data, fsevent_results) = FsEvents::get_fsevent_data(input_results, &sig)?;
            input_results = input_data;
            fsevents_array.push(fsevent_results);
            if input_results.len() == 0 {
                break;
            }
        }

        Ok((input_results, fsevents_array))
    }

    /// Parse FsEvent header
    fn fsevents_header(data: &[u8]) -> nom::IResult<&[u8], FsEventsHeader> {
        let mut fsevent = FsEventsHeader {
            signature: 0,
            padding: 0,
            stream_size: 0,
        };

        let (input, sig) = take(size_of::<u32>())(data)?;
        let (input, padding) = take(size_of::<u32>())(input)?;
        let (input, stream_size) = take(size_of::<u32>())(input)?;

        let (_, fsevent_sig) = le_u32(sig)?;
        let (_, fsevent_pad) = le_u32(padding)?;
        let (_, fsevent_stream) = le_u32(stream_size)?;

        fsevent.signature = fsevent_sig;
        fsevent.padding = fsevent_pad;
        fsevent.stream_size = fsevent_stream;

        Ok((input, fsevent))
    }

    /// Parse FsEvent stream entry
    fn get_fsevent_data<'a>(data: &'a [u8], sig: &u32) -> nom::IResult<&'a [u8], FsEvents> {
        let mut fsevent_data = FsEvents {
            flags: String::new(),
            path: String::from("/"),
            node: 0,
            event_id: 0,
        };

        // Read path until end-of-string character
        let (input, path) = nom::bytes::streaming::take_while(|b: u8| b != 0)(data)?;
        // Nom end-of-string character
        let (input, _) = nom::bytes::streaming::take(size_of::<u8>())(input)?;
        let (input, id) = nom::bytes::streaming::take(size_of::<u64>())(input)?;
        let (input, flags) = nom::bytes::streaming::take(size_of::<u32>())(input)?;

        let (_, fsevent_id) = le_u64(id)?;
        let (_, fsevent_flags) = le_u32(flags)?;

        let flag_list = FsEvents::match_flags(&fsevent_flags);

        fsevent_data.flags = flag_list.join(",").to_string();
        fsevent_data.event_id = fsevent_id;
        fsevent_data.path += from_utf8(&path.to_vec()).unwrap_or_default();

        if fsevent_data.path.starts_with("//") {
            fsevent_data.path = (&fsevent_data.path[1..]).to_string();
        }

        if sig != &FsEvents::DISKLOGGERV1 {
            let (input, node) = nom::bytes::streaming::take(size_of::<u64>())(input)?;
            let (_, fsevent_node) = le_u64(node)?;

            fsevent_data.node = fsevent_node;
            return Ok((input, fsevent_data));
        }

        Ok((input, fsevent_data))
    }

    /// Identify Event flags in FsEvent entry
    fn match_flags(flags: &u32) -> Vec<String> {
        let mut flag_list: Vec<String> = Vec::new();
        if (flags & 0x0) != 0 {
            flag_list.push("None".to_string());
        }
        if (flags & 0x01) != 0 {
            flag_list.push("Created".to_string());
        }
        if (flags & 0x02) != 0 {
            flag_list.push("Removed".to_string());
        }
        if (flags & 0x04) != 0 {
            flag_list.push("InodeMetadataModified".to_string());
        }
        if (flags & 0x08) != 0 {
            flag_list.push("Renamed".to_string());
        }
        if (flags & 0x10) != 0 {
            flag_list.push("Modified".to_string());
        }
        if (flags & 0x20) != 0 {
            flag_list.push("Exchange".to_string());
        }
        if (flags & 0x40) != 0 {
            flag_list.push("FinderInfoModified".to_string());
        }
        if (flags & 0x80) != 0 {
            flag_list.push("DirectoryCreated".to_string());
        }
        if (flags & 0x100) != 0 {
            flag_list.push("PermissionChanged".to_string());
        }
        if (flags & 0x200) != 0 {
            flag_list.push("ExtendedAttributeModified".to_string());
        }
        if (flags & 0x400) != 0 {
            flag_list.push("ExtenedAttributeRemoved".to_string());
        }
        if (flags & 0x800) != 0 {
            flag_list.push("DocumentCreated".to_string());
        }
        if (flags & 0x1000) != 0 {
            flag_list.push("DocumentRevision".to_string());
        }
        if (flags & 0x2000) != 0 {
            flag_list.push("UnmountPending".to_string());
        }
        if (flags & 0x4000) != 0 {
            flag_list.push("ItemCloned".to_string());
        }
        if (flags & 0x10000) != 0 {
            flag_list.push("NotificationClone".to_string());
        }
        if (flags & 0x20000) != 0 {
            flag_list.push("ItemTruncated".to_string());
        }
        if (flags & 0x40000) != 0 {
            flag_list.push("DirectoryEvent".to_string());
        }
        if (flags & 0x80000) != 0 {
            flag_list.push("LastHardLinkRemoved".to_string());
        }
        if (flags & 0x100000) != 0 {
            flag_list.push("IsHardLink".to_string());
        }
        if (flags & 0x400000) != 0 {
            flag_list.push("IsSymbolicLink".to_string());
        }
        if (flags & 0x800000) != 0 {
            flag_list.push("IsFile".to_string());
        }
        if (flags & 0x1000000) != 0 {
            flag_list.push("IsDirectory".to_string());
        }
        if (flags & 0x2000000) != 0 {
            flag_list.push("Mount".to_string());
        }
        if (flags & 0x4000000) != 0 {
            flag_list.push("Unmount".to_string());
        }
        if (flags & 0x20000000) != 0 {
            flag_list.push("EndOfTransaction".to_string());
        }
        return flag_list;
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::PathBuf};

    use crate::parser::decompress;

    use super::FsEvents;

    #[test]
    fn test_match_flags() {
        let data: u32 = 11;
        let results = FsEvents::match_flags(&data);
        assert!(results[0] == "Created");
        assert!(results[1] == "Removed");
        assert!(results[2] == "Renamed");
    }

    #[test]
    fn test_fsevents_data() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/DLS2/0000000000027d79");
        let test_path: &str = &test_location.display().to_string();
        let files = decompress(test_path).unwrap();
        let (results, data) = FsEvents::fsevents_data(&files).unwrap();
        assert!(results.len() == 0);
        assert!(data.len() == 736);
    }

    #[test]
    fn test_fsevents_headers() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/Headers/dls2header");
        let mut open = File::open(test_location).unwrap();
        let mut buffer = Vec::new();
        open.read_to_end(&mut buffer).unwrap();
        let (_, header) = FsEvents::fsevents_header(&buffer).unwrap();
        assert!(header.signature == 1145852722);
        assert!(header.padding == 779163104);
        assert!(header.stream_size == 78970);
    }

    #[test]
    fn test_get_fsevent_data() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/Uncompressed/0000000000027d79");
        let mut open = File::open(test_location).unwrap();
        let mut buffer = Vec::new();
        open.read_to_end(&mut buffer).unwrap();
        let (input, header) = FsEvents::fsevents_header(&buffer).unwrap();

        let (_, results) = FsEvents::get_fsevent_data(input, &header.signature).unwrap();

        assert!(results.event_id == 163140);
        assert!(results.path == "/Volumes/Preboot");
        assert!(results.node == 0);
        assert!(results.flags == "Removed,IsDirectory,Mount,Unmount");
    }

    #[test]
    fn test_get_fsevent() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/Uncompressed/0000000000027d79");
        let mut open = File::open(test_location).unwrap();
        let mut buffer = Vec::new();
        open.read_to_end(&mut buffer).unwrap();
        let (input, header) = FsEvents::fsevents_header(&buffer).unwrap();

        let (input, results) = FsEvents::get_fsevent(input, header.signature).unwrap();
        assert!(results.len() == 736);
        assert!(input.len() == 0);
    }
}
