use super::gb_mmu::{Address, Data, MemoryReadError, MemoryWriteError};
use mbc::Mbc;

mod mbc;

const MBC_TYPE_ADDR: Address = 0x0147;
const MIN_CARTRIDGE_SIZE: usize = 32768;
const CARTRIDGE_HEADER_SIZE: usize = 0x014F - 0x0100 + 1;
const ENTRY_POINT_SIZE: usize = 0x0103 - 0x0100 + 1;
const LOGO_SIZE: usize = 0x0133 - 0x0104 + 1;

#[derive(Debug, Default)]
enum CgbFlag {
    #[default]
    CgbEnhanced,
    CgbOnly,
}

impl From<Data> for CgbFlag {
    fn from(data: Data) -> Self {
        match data {
            0xC0 => Self::CgbOnly,
            _ => Self::CgbEnhanced,
        }
    }
}

#[derive(Debug, Default)]
enum NewLicenseeCode {
    #[default]
    None,
    NintendoResearchAndDevelopment1,
    Capcom,
    ElectronicArts,
    HudsonSoft,
    BAi,
    Kss,
    PlanningOfficeWada,
    PcmComplete,
    SanX,
    Kemco,
    SetaCorporation,
    Viacom,
    Nintendo,
    Bandai,
    OceanSoftwareAcclaimEntertainment,
    Konami,
    HectorSoft,
    Taito,
    // HudsonSoft,
    Banpresto,
    Ubisoft,
    Atlus,
    MalibuInteractive,
    Angel,
    BulletProofSoftware,
    Irem,
    Absolute,
    AcclaimEntertainment,
    Activision,
    SammyUsaCorporation,
    // Konami,
    HiTechExpressions,
    Ljn,
    Matchbox,
    Mattel,
    MiltonBradleyCompany,
    TitusInteractive,
    VirginGamesLtd,
    LucasfilmGames,
    OceanSoftware,
    // ElectronicArts,
    Infogrames,
    InterplayEntertainment,
    Broderbund,
    SculpturedSoftware,
    TheSalesCurveLimited,
    Thq,
    Accolade,
    MisawaEntertainment,
    LozcG,
    TokumaShoten,
    TsukudaOriginal,
    Chunsoft,
    VideoSystem,
    // OceanSoftwareAcclaimEntertainment,
    Varie,
    Yonezawa,
    Kaneko,
    PackInVideo,
    BottomUp,
    KonamiYuGiOh,
    Mto,
    Kodansha,
}

impl TryFrom<&[Data]> for NewLicenseeCode {
    type Error = String;

    fn try_from(data: &[Data]) -> Result<Self, Self::Error> {
        match data {
            b"00" => Ok(Self::None),
            b"01" => Ok(Self::NintendoResearchAndDevelopment1),
            // TODO: Handle the rest of the enum variants here
            _ => Err(format!(
                "Unknown code: {:#X?}, unable to marshal into NewLicenseeCode",
                data
            )),
        }
    }
}

#[derive(Debug, Default)]
enum SgbFlag {
    SgbSupported,
    #[default]
    SgbUnsupported,
}

impl From<Data> for SgbFlag {
    fn from(data: Data) -> Self {
        match data {
            0x03 => Self::SgbSupported,
            _ => Self::SgbUnsupported,
        }
    }
}

#[derive(Debug, Default)]
enum CartridgeType {
    #[default]
    RomOnly,
    Mbc1,
    // TODO: Fill out the rest of the enum variants here
}

impl TryFrom<Data> for CartridgeType {
    type Error = String;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        match data {
            0x00 => Ok(Self::RomOnly),
            0x01 => Ok(Self::Mbc1),
            // TODO: Handle the rest of the enum variants here
            _ => Err(format!(
                "Unknown code: {:#X}, unable to marshal into CartridgeType",
                data
            )),
        }
    }
}

#[derive(Debug, Default)]
enum RomSize {
    #[default]
    // ROM Banks: 2 (no banking)
    Size32KiB,

    // ROM Banks: 4
    Size64KiB,

    // ROM Banks: 8
    Size128KiB,

    // ROM Banks: 16
    Size256KiB,

    // ROM Banks: 32
    Size512KiB,

    // ROM Banks: 64
    Size1MiB,

    // ROM Banks: 128
    Size2MiB,

    // ROM Banks: 256
    Size4MiB,

    // ROM Banks: 512
    Size8MiB,

    // ROM Banks: 72
    Size1_1MiB,

    // ROM Banks: 80
    Size1_2MiB,

    // ROM Banks: 96
    Size1_5MiB,
}

impl TryFrom<Data> for RomSize {
    type Error = String;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        match data {
            0x00 => Ok(Self::Size32KiB),
            0x01 => Ok(Self::Size64KiB),
            0x02 => Ok(Self::Size128KiB),
            0x03 => Ok(Self::Size256KiB),
            0x04 => Ok(Self::Size512KiB),
            0x05 => Ok(Self::Size1MiB),
            0x06 => Ok(Self::Size2MiB),
            0x07 => Ok(Self::Size4MiB),
            0x08 => Ok(Self::Size8MiB),
            0x52 => Ok(Self::Size1_1MiB),
            0x53 => Ok(Self::Size1_2MiB),
            0x54 => Ok(Self::Size1_5MiB),
            _ => Err(format!(
                "Unknown code: {:#X}, unable to marshal into RomSize",
                data
            )),
        }
    }
}

#[derive(Debug, Default)]
enum RamSize {
    #[default]
    // RAM Banks: No RAM
    Size0KiB,

    // RAM Banks: Unused (2 KiB chips never used in real cartridges)
    Size2KiB,

    // RAM Banks: 1
    Size8KiB,

    // RAM Banks: 4
    Size32KiB,

    // RAM Banks: 16
    Size128KiB,

    // RAM Banks: 8
    Size64KiB,
}

impl TryFrom<Data> for RamSize {
    type Error = String;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        match data {
            0x00 => Ok(Self::Size0KiB),
            0x01 => Ok(Self::Size2KiB),
            0x02 => Ok(Self::Size8KiB),
            0x03 => Ok(Self::Size32KiB),
            0x04 => Ok(Self::Size128KiB),
            0x05 => Ok(Self::Size64KiB),
            _ => Err(format!(
                "Unknown code: {:#X}, unable to marshal into RamSize",
                data
            )),
        }
    }
}

#[derive(Debug, Default)]
enum DestinationCode {
    #[default]
    JapanAndOverseas,
    OverseasOnly,
}

impl TryFrom<Data> for DestinationCode {
    type Error = String;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        match data {
            0x00 => Ok(Self::JapanAndOverseas),
            0x01 => Ok(Self::OverseasOnly),
            _ => Err(format!(
                "Unknown code: {:#X}, unable to marshal into DestinationCode",
                data
            )),
        }
    }
}

#[derive(Debug, Default)]
enum OldLicenseeCode {
    #[default]
    None,
    Nintendo,
    // TODO: Fill out the rest of the enum variants here
}

impl TryFrom<Data> for OldLicenseeCode {
    type Error = String;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        match data {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Nintendo),
            // TODO: Handle the rest of the enum variants here
            _ => Err(format!(
                "Unknown code: {:#X}, unable to marshal into OldLicenseeCode",
                data
            )),
        }
    }
}

#[derive(Debug)]
struct CartridgeHeader {
    // [0x0100, 0x0103]
    entry_point: [Data; ENTRY_POINT_SIZE],

    // [0x0104, 0x0133]
    logo: [Data; LOGO_SIZE],

    // [0x0134, (0x0143 or 0x0142 or 0x013E)]
    title: String,

    // [0x013F, 0x0142]
    manufacturer_code: Option<String>,

    // 0x0143
    cgb_flag: CgbFlag,

    // [0x0144, 0x0145]
    new_licensee_code: Option<NewLicenseeCode>,

    // 0x0146
    sgb_flag: SgbFlag,

    // 0x0147
    cartridge_type: CartridgeType,

    // 0x0148
    rom_size: RomSize,

    // 0x0149
    ram_size: RamSize,

    // 0x014A
    destination_code: DestinationCode,

    // 0x014B
    old_licensee_code: OldLicenseeCode,

    // 0x014C
    version_number: Data,

    // 0x014D
    header_checksum: Data,

    // [0x014E, 0x014F]
    global_checksum: u16,
}

impl CartridgeHeader {
    fn parse_title(data: &[Data]) -> String {
        // Find the first index where the byte is 0x00 all bytes after that are assumed to be 0x00
        // let end_idx = data.iter().position(|&byte| byte == 0x00).unwrap_or(data.len());

        // std::str::from_utf8(&data[..end_idx])
        //     .map(|result| result.to_string())
        //     .unwrap_or_else(|_| data[..end_idx].iter().map(|&byte| byte as char).collect())

        // String::from_utf8_lossy(&data[..end_idx]).into_owned()

        // Titles are null padded
        String::from_utf8_lossy(data)
            .trim_end_matches(char::from(0x00))
            .to_owned()
    }
}

impl From<&[Data]> for CartridgeHeader {
    fn from(data: &[Data]) -> Self {
        Self {
            entry_point: data[0x0000..=ENTRY_POINT_SIZE]
                .try_into()
                .unwrap_or_else(|_| {
                    panic!(
                        "Entry point must be at least {} bytes long",
                        ENTRY_POINT_SIZE
                    )
                }),
            logo: data[ENTRY_POINT_SIZE..=(ENTRY_POINT_SIZE + LOGO_SIZE)]
                .try_into()
                .unwrap_or_else(|_| panic!("Logo must be at least {} bytes long", LOGO_SIZE)),
            title: Self::parse_title(&data[0x0034..=0x0043]),
            manufacturer_code: todo!(),
            cgb_flag: todo!(),
            new_licensee_code: todo!(),
            sgb_flag: todo!(),
            cartridge_type: todo!(),
            rom_size: todo!(),
            ram_size: todo!(),
            destination_code: todo!(),
            old_licensee_code: todo!(),
            version_number: todo!(),
            header_checksum: todo!(),
            global_checksum: todo!(),
        }
    }
}

impl Default for CartridgeHeader {
    fn default() -> Self {
        Self {
            entry_point: [0x00; ENTRY_POINT_SIZE],
            logo: [0x00; LOGO_SIZE],
            title: Default::default(),
            manufacturer_code: Default::default(),
            cgb_flag: Default::default(),
            new_licensee_code: Default::default(),
            sgb_flag: Default::default(),
            cartridge_type: Default::default(),
            rom_size: Default::default(),
            ram_size: Default::default(),
            destination_code: Default::default(),
            old_licensee_code: Default::default(),
            version_number: Default::default(),
            header_checksum: Default::default(),
            global_checksum: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct GbCartridge {
    header: CartridgeHeader,
    mbc: Mbc,
}

impl GbCartridge {
    // TODO: Fill out methods for GbCartridge here

    pub fn read(&self, addr: Address) -> Result<Data, MemoryReadError> {
        todo!();
    }

    pub fn write(&mut self, addr: Address, data: Data) -> Result<(), MemoryWriteError> {
        todo!();
    }

    pub fn reset(&mut self) {
        todo!();
    }
}

#[derive(Debug)]
pub enum CartridgeError {
    // Cartridge memory size is smaller than the minimum (32 KiB)
    TooSmall(usize),
}

impl TryFrom<&[Data]> for GbCartridge {
    type Error = CartridgeError;

    fn try_from(data: &[Data]) -> Result<Self, Self::Error> {
        if data.len() < MIN_CARTRIDGE_SIZE {
            return Err(CartridgeError::TooSmall(data.len()));
        }

        Ok(Self {
            header: data[0x0100..=0x014F].into(),
            // TODO: Default construct the rest of the struct fields for now
            ..Default::default()
        })
    }
}
