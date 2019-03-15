#[macro_use]
extern crate bitflags;

pub mod sysinfo;

#[derive(Default)]
pub struct LoggingInfo {
    pub host_serial: Option<String>,
    pub drive_serial: Option<String>,
    pub drive_product: Option<String>,
}

impl LoggingInfo {
    pub fn new() -> LoggingInfo {
        Default::default()
    }
}

pub mod ewfargs {
    #[derive(Copy, Clone)]
    pub enum NumSectors {
        Sectors16,
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

    impl Default for NumSectors {
        fn default() -> Self {
            NumSectors::Sectors64
        }
    }

    #[derive(Copy, Clone)]
    pub enum CompressionType {
        None, // default
        EmptyBlock,
        Fast,
        Best,
    }

    impl Default for CompressionType {
        fn default() -> Self {
            CompressionType::None
        }
    }

    bitflags! {
        #[derive(Default)]
        pub struct DigestType: u8 {
            const MD5 = 0; // mandatory
            const SHA1 = 1; // 1 << 0
            const SHA256 = (1 << 1);
        }
    }

    #[derive(Copy, Clone)]
    pub enum EwfFormat {
        FTK,
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

    impl Default for EwfFormat {
        fn default() -> Self {
            EwfFormat::Encase6
        }
    }

    /// Stores arguments to pass to ewfacquirestream.
    #[derive(Clone, Default)]
    pub struct ArgsList {
        /// The device to image.
        pub source_device: Option<String>, // set in windows::select_source
        /// The number of sectors to read at once.
        pub num_sectors: NumSectors, // TODO
        /// The type of compression to use.
        pub compression_type: CompressionType, // set in windows::target_info_next
        /// The case number.
        pub case_number: Option<String>, // set in windows::examiner_info_next
        /// Which digests to calculate (MD5 is required).
        pub digest_type: DigestType, // set in windows::target_info_next
        /// The description of the evidence.
        pub description: Option<String>, // set in windows::examiner_info_next
        /// The examiner's name.
        pub examiner_name: Option<String>, // set in windows::examiner_info_next
        /// The evidence number for the evidence being imaged.
        pub evidence_number: Option<String>, // set in windows::examiner_info_next
        /// Which file format to use for the images.
        pub ewf_format: EwfFormat, // set in windows::target_info_next
        /// Notes about the evidence.
        pub notes: Option<String>, // set in windows::examiner_info_next
        /// How many bytes are in a sector.
        pub bytes_per_sector: Option<i32>, // TODO
        /// How large to make segments for segmented image files.
        pub segment_file_size: Option<String>, // set in windows::target_info_next
        /// The path to the target.
        pub target_dir: Option<String>, // set in windows::target_info_next
        /// The path to the seconday target.
        pub secondary_target_dir: Option<String>, // set in windows::target_info_next
        /// The filename to use for both the target and the secondary target.
        pub target_filename: Option<String>, // set in windows::target_info_next
    }

    impl ArgsList {
        /// Returns a new `ArgsList` with all default options.
        pub fn new() -> ArgsList {
            Default::default()
        }
    }
}
