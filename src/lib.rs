#[macro_use]
extern crate bitflags;

pub mod sysinfo;

pub struct LoggingInfo {
    pub host_serial: Option<String>,
    pub drive_serial: Option<String>,
    pub drive_product: Option<String>,
}

impl LoggingInfo {
    pub fn new() -> LoggingInfo {
        LoggingInfo {
            host_serial: None,
            drive_serial: None,
            drive_product: None,
        }
    }
}

pub mod ewfargs {
    #[derive(Copy, Clone)]
    pub enum NumSectors {
        Sectors16 = -2,
        Sectors32,
        Sectors64, // default
        Sectors128,
        Sectors256,
        Sectors512,
        Sectors1024,
        Sectors2048,
        Sectors4096,
        Sectors8192,
        Sectors16384,
        Sectors32768,
    }

    #[derive(Copy, Clone)]
    pub enum CompressionType {
        None = 0, // default
        EmptyBlock,
        Fast,
        Best,
    }

    bitflags! {
        pub struct DigestType: u8 {
            const MD5 = 0; // mandatory
            const SHA1 = (1 << 0);
            const SHA512 = (1 << 1);
        }
    }

    #[derive(Copy, Clone)]
    pub enum EwfFormat {
        FTK = -5,
        Encase2,
        Encase3,
        Encase4,
        Encase5,
        Encase6, // default
        Encase7,
        Linen5,
        Linen6,
        Linen7,
        EwfX,
    }

    /// Stores arguments to pass to ewfacquirestream.
    #[derive(Clone)]
    pub struct ArgsList {
        /// The device to image.
        pub source_device: Option<String>,
        /// The number of sectors to read at once.
        pub num_sectors: NumSectors,
        /// The type of compression to use.
        pub compression_type: CompressionType,
        /// The case number.
        pub case_number: Option<String>,
        /// Which digests to calculate (MD5 is required).
        pub digest_type: DigestType,
        /// The description of the evidence.
        pub description: Option<String>,
        /// The examiner's name.
        pub examiner_name: Option<String>,
        /// The evidence number for the evidence being imaged.
        pub evidence_number: Option<String>,
        /// Which file format to use for the images.
        pub ewf_format: EwfFormat,
        /// Notes about the evidence.
        pub notes: Option<String>,
        /// How many bytes are in a sector.
        pub bytes_per_sector: Option<i32>,
        /// How large to make segments for segmented image files.
        pub segment_file_size: Option<String>,
        /// The path to the target.
        pub target: Option<String>,
        /// The path to the seconday target.
        pub secondary_target: Option<String>,
    }

    impl ArgsList {
        /// Returns a new `ArgsList` with all default options.
        pub fn new() -> ArgsList {
            ArgsList {
                source_device: None,
                num_sectors: NumSectors::Sectors64,
                compression_type: CompressionType::None,
                case_number: None,
                digest_type: DigestType::MD5,
                description: None,
                examiner_name: None,
                evidence_number: None,
                ewf_format: EwfFormat::Encase6,
                notes: None,
                bytes_per_sector: None,
                segment_file_size: None,
                target: None,
                secondary_target: None,
            }
        }
    }
}
