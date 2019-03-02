pub mod ewfargs {
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

    pub enum CompressionType {
        None = 0, // default
        EmptyBlock,
        Fast,
        Best,
    }

    pub enum DigestType {
        MD5 = 0, // mandatory
        SHA1 = (1 << 0),
        SHA512 = (1 << 1),
    }

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

    pub struct ArgsList {
        pub source_device: Option<String>,
        pub num_sectors: Option<NumSectors>,
        pub compression_type: Option<CompressionType>,
        pub case_number: Option<String>,
        pub digest_type: Option<DigestType>,
        pub description: Option<String>,
        pub examiner_name: Option<String>,
        pub evidence_number: Option<String>,
        pub ewf_format: Option<EwfFormat>,
        pub notes: Option<String>,
        pub bytes_per_sector: Option<i32>,
        pub segment_file_size: Option<String>,
        pub target: Option<String>,
        pub secondary_target: Option<String>,
    }

    impl ArgsList {
        pub fn new() -> ArgsList {
            ArgsList {
                source_device: None,
                num_sectors: None,
                compression_type: None,
                case_number: None,
                digest_type: None,
                description: None,
                examiner_name: None,
                evidence_number: None,
                ewf_format: None,
                notes: None,
                bytes_per_sector: None,
                segment_file_size: None,
                target: None,
                secondary_target: None,
            }
        }
    }
}
