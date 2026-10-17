use super::gb_mmu::{Address, Data, MemoryReadError, MemoryWriteError};
use mbc::Mbc;
use tracing::info;

mod mbc;

const MBC_TYPE_ADDR: Address = 0x0147;
const MIN_CARTRIDGE_SIZE: usize = 32768;
const CARTRIDGE_HEADER_SIZE: usize = 0x014F - 0x0100 + 1;
const CARTRIDGE_HEADER_START_ADDR: Address = 0x0100;
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
                "Unknown data: {:#X?}, unable to deserialize into NewLicenseeCode",
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
                "Unknown data: {:#X}, unable to deserialize into CartridgeType",
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
                "Unknown data: {:#X}, unable to deserialize into RomSize",
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
                "Unknown data: {:#X}, unable to deserialize into RamSize",
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
                "Unknown data: {:#X}, unable to deserialize into DestinationCode",
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
    Capcom,
    HotB,
    Jaleco,
    CoconutsJapan,
    EliteSystems,
    Ea,
    HudsonSoft,
    ItcEntertainment,
    Yanoman,
    JapanClary,
    VirginGames,
    PcmComplete,
    SanX,
    Kemco,
    SetaCorporation,
    Infogrames,
    // Nintendo,
    Bandai,
    NewLicenseeCode,
    Konami,
    HectorSoft,
    // Capcom,
    Banpresto,
    EntertainmentInteractive,
    Gremlin,
    Ubisoft,
    Atlus,
    MalibuInteractive,
    Angel,
    SpectrumHoloByte,
    Irem,
    // VirginGames,
    // MalibuInteractive,
    UsGold,
    Absolute,
    AcclaimEntertainment,
    Activision,
    SammyCorporation,
    GameTek,
    ParkPlace,
    Ljn,
    Matchbox,
    MiltonBradleyCompany,
    Mindscape,
    Romstar,
    NaxatSoft,
    Tradewest,
    TitusInteractive,
    // VirginGames,
    OceanSoftware,
    // Ea,
    // EliteSystems,
    ElectroBrain,
    // Infogrames,
    InterplayEntertainment,
    Broderbund,
    SculpturedSoftware,
    TheSalesCurveLimited,
    Thq,
    Accolade,
    TriffixEntertainment,
    MicroProse,
    // Kemco,
    MisawaEntertainment,
    LozcG,
    TokumaShoten,
    BulletProofSoftware,
    VicTokaiCorp,
    ApeInc,
    IMax,
    ChunsoftCo,
    VideoSystem,
    TsubarayaProductions,
    Varie,
    Yonezawa,
    // Kemco,
    Arc,
    NihonBussan,
    Tecmo,
    Imagineer,
    // Banpresto,
    Nova,
    HoriElectric,
    // Bandai,
    // Konami,
    Kawada,
    Takara,
    TechnosJapan,
    // Broderbund,
    ToeiAnimation,
    Toho,
    Namco,
    // AcclaimEntertainment,
    AsciiCorporation,
    // Bandai,
    SquareEnix,
    HalLaboratory,
    Snk,
    PonyCanyon,
    CultureBrain,
    Sunsoft,
    SonyImagesoft,
    // SammyCorporation,
    Taito,
    // Kemco,
    Square,
    // TokumaShoten,
    DataEast,
    TonkinHouse,
    Koei,
    Ufl,
    UltraGames,
    VapInc,
    UseCorporation,
    Meldac,
    // PonyCanyon,
    // Angel,
    // Taito,
    Sofel,
    Quest,
    SigmaEnterprises,
    AskKodanshaCo,
    // NaxatSoft,
    CopyaSystem,
    // Banpresto,
    Tomy,
    // Ljn,
    NipponComputerSystems,
    HumanEnt,
    Altron,
    // Jaleco,
    TowaChiki,
    Yutaka,
    // Varie,
    Epoch,
    Athena,
    AsmikAceEntertainment,
    Natsume,
    KingRecords,
    // Atlus,
    SonyRecords,
    Igs,
    AWave,
    ExtremeEntertainment,
    // Ljn,
}

impl TryFrom<Data> for OldLicenseeCode {
    type Error = String;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        match data {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Nintendo),
            0x08 => Ok(Self::Capcom),
            0x09 => Ok(Self::HotB),
            0x0A => Ok(Self::Jaleco),
            0x0B => Ok(Self::CoconutsJapan),
            0x0C => Ok(Self::EliteSystems),
            0x13 => Ok(Self::Ea),
            0x18 => Ok(Self::HudsonSoft),
            0x19 => Ok(Self::ItcEntertainment),
            0x1A => Ok(Self::Yanoman),
            0x1D => Ok(Self::JapanClary),
            0x1F => Ok(Self::VirginGames),
            0x24 => Ok(Self::PcmComplete),
            0x25 => Ok(Self::SanX),
            0x28 => Ok(Self::Kemco),
            0x29 => Ok(Self::SetaCorporation),
            0x30 => Ok(Self::Infogrames),
            0x31 => Ok(Self::Nintendo),
            0x32 => Ok(Self::Bandai),
            0x33 => Ok(Self::NewLicenseeCode),
            0x34 => Ok(Self::Konami),
            0x35 => Ok(Self::HectorSoft),
            0x38 => Ok(Self::Capcom),
            0x39 => Ok(Self::Banpresto),
            0x3C => Ok(Self::EntertainmentInteractive),
            0x3E => Ok(Self::Gremlin),
            0x41 => Ok(Self::Ubisoft),
            0x42 => Ok(Self::Atlus),
            0x44 => Ok(Self::MalibuInteractive),
            0x46 => Ok(Self::Angel),
            0x47 => Ok(Self::SpectrumHoloByte),
            0x49 => Ok(Self::Irem),
            0x4A => Ok(Self::VirginGames),
            0x4D => Ok(Self::MalibuInteractive),
            0x4F => Ok(Self::UsGold),
            0x50 => Ok(Self::Absolute),
            0x51 => Ok(Self::AcclaimEntertainment),
            0x52 => Ok(Self::Activision),
            0x53 => Ok(Self::SammyCorporation),
            0x54 => Ok(Self::GameTek),
            0x55 => Ok(Self::ParkPlace),
            0x56 => Ok(Self::Ljn),
            0x57 => Ok(Self::Matchbox),
            0x59 => Ok(Self::MiltonBradleyCompany),
            0x5A => Ok(Self::Mindscape),
            0x5B => Ok(Self::Romstar),
            0x5C => Ok(Self::NaxatSoft),
            0x5D => Ok(Self::Tradewest),
            0x60 => Ok(Self::TitusInteractive),
            0x61 => Ok(Self::VirginGames),
            0x67 => Ok(Self::OceanSoftware),
            0x69 => Ok(Self::Ea),
            0x6E => Ok(Self::EliteSystems),
            0x6F => Ok(Self::ElectroBrain),
            0x70 => Ok(Self::Infogrames),
            0x71 => Ok(Self::InterplayEntertainment),
            0x72 => Ok(Self::Broderbund),
            0x73 => Ok(Self::SculpturedSoftware),
            0x75 => Ok(Self::TheSalesCurveLimited),
            0x78 => Ok(Self::Thq),
            0x79 => Ok(Self::Accolade),
            0x7A => Ok(Self::TriffixEntertainment),
            0x7C => Ok(Self::MicroProse),
            0x7F => Ok(Self::Kemco),
            0x80 => Ok(Self::MisawaEntertainment),
            0x83 => Ok(Self::LozcG),
            0x86 => Ok(Self::TokumaShoten),
            0x8B => Ok(Self::BulletProofSoftware),
            0x8C => Ok(Self::VicTokaiCorp),
            0x8E => Ok(Self::ApeInc),
            0x8F => Ok(Self::IMax),
            0x91 => Ok(Self::ChunsoftCo),
            0x92 => Ok(Self::VideoSystem),
            0x93 => Ok(Self::TsubarayaProductions),
            0x95 => Ok(Self::Varie),
            0x96 => Ok(Self::Yonezawa),
            0x97 => Ok(Self::Kemco),
            0x99 => Ok(Self::Arc),
            0x9A => Ok(Self::NihonBussan),
            0x9B => Ok(Self::Tecmo),
            0x9C => Ok(Self::Imagineer),
            0x9D => Ok(Self::Banpresto),
            0x9F => Ok(Self::Nova),
            0xA1 => Ok(Self::HoriElectric),
            0xA2 => Ok(Self::Bandai),
            0xA4 => Ok(Self::Konami),
            0xA6 => Ok(Self::Kawada),
            0xA7 => Ok(Self::Takara),
            0xA9 => Ok(Self::TechnosJapan),
            0xAA => Ok(Self::Broderbund),
            0xAC => Ok(Self::ToeiAnimation),
            0xAD => Ok(Self::Toho),
            0xAF => Ok(Self::Namco),
            0xB0 => Ok(Self::AcclaimEntertainment),
            0xB1 => Ok(Self::AsciiCorporation),
            0xB2 => Ok(Self::Bandai),
            0xB4 => Ok(Self::SquareEnix),
            0xB6 => Ok(Self::HalLaboratory),
            0xB7 => Ok(Self::Snk),
            0xB9 => Ok(Self::PonyCanyon),
            0xBA => Ok(Self::CultureBrain),
            0xBB => Ok(Self::Sunsoft),
            0xBD => Ok(Self::SonyImagesoft),
            0xBF => Ok(Self::SammyCorporation),
            0xC0 => Ok(Self::Taito),
            0xC2 => Ok(Self::Kemco),
            0xC3 => Ok(Self::Square),
            0xC4 => Ok(Self::TokumaShoten),
            0xC5 => Ok(Self::DataEast),
            0xC6 => Ok(Self::TonkinHouse),
            0xC8 => Ok(Self::Koei),
            0xC9 => Ok(Self::Ufl),
            0xCA => Ok(Self::UltraGames),
            0xCB => Ok(Self::VapInc),
            0xCC => Ok(Self::UseCorporation),
            0xCD => Ok(Self::Meldac),
            0xCE => Ok(Self::PonyCanyon),
            0xCF => Ok(Self::Angel),
            0xD0 => Ok(Self::Taito),
            0xD1 => Ok(Self::Sofel),
            0xD2 => Ok(Self::Quest),
            0xD3 => Ok(Self::SigmaEnterprises),
            0xD4 => Ok(Self::AskKodanshaCo),
            0xD6 => Ok(Self::NaxatSoft),
            0xD7 => Ok(Self::CopyaSystem),
            0xD9 => Ok(Self::Banpresto),
            0xDA => Ok(Self::Tomy),
            0xDB => Ok(Self::Ljn),
            0xDD => Ok(Self::NipponComputerSystems),
            0xDE => Ok(Self::HumanEnt),
            0xDF => Ok(Self::Altron),
            0xE0 => Ok(Self::Jaleco),
            0xE1 => Ok(Self::TowaChiki),
            0xE2 => Ok(Self::Yutaka),
            0xE3 => Ok(Self::Varie),
            0xE5 => Ok(Self::Epoch),
            0xE7 => Ok(Self::Athena),
            0xE8 => Ok(Self::AsmikAceEntertainment),
            0xE9 => Ok(Self::Natsume),
            0xEA => Ok(Self::KingRecords),
            0xEB => Ok(Self::Atlus),
            0xEC => Ok(Self::SonyRecords),
            0xEE => Ok(Self::Igs),
            0xF0 => Ok(Self::AWave),
            0xF3 => Ok(Self::ExtremeEntertainment),
            0xFF => Ok(Self::Ljn),
            _ => Err(format!(
                "Unknown data: {:#X}, unable to deserialize into OldLicenseeCode",
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
    fn parse_entry_point(data: &[Data]) -> [Data; ENTRY_POINT_SIZE] {
        data.try_into().unwrap_or_else(|_| {
            panic!(
                "Data was {} bytes long, entry point must be {} bytes long",
                data.len(),
                ENTRY_POINT_SIZE
            )
        })
    }

    fn parse_logo(data: &[Data]) -> [Data; LOGO_SIZE] {
        data.try_into().unwrap_or_else(|_| {
            panic!(
                "Data was {} bytes long, logo must be {} bytes long",
                data.len(),
                LOGO_SIZE
            )
        })
    }

    fn parse_title(data: &[Data]) -> String {
        // Titles are null padded
        String::from_utf8_lossy(data)
            .trim_end_matches(char::from(0x00))
            .to_owned()
    }

    fn parse_manufacturer_code(data: &[Data]) -> Option<String> {
        todo!()
    }

    fn parse_cgb_flag(data: Data) -> CgbFlag {
        todo!()
    }

    fn parse_new_licensee_code(data: &[Data]) -> Option<NewLicenseeCode> {
        todo!()
    }

    fn parse_sgb_flag(data: Data) -> SgbFlag {
        todo!()
    }

    fn parse_cartridge_type(data: Data) -> CartridgeType {
        todo!()
    }

    fn parse_rom_size(data: Data) -> RomSize {
        todo!()
    }

    fn parse_ram_size(data: Data) -> RamSize {
        todo!()
    }

    fn parse_destination_code(data: Data) -> DestinationCode {
        todo!()
    }

    fn parse_old_licensee_code(data: Data) -> OldLicenseeCode {
        data.try_into().unwrap_or_else(|err| panic!("{}", err))
    }

    fn parse_version_number(data: Data) -> Data {
        todo!()
    }

    fn parse_header_checksum(data: Data) -> Data {
        todo!()
    }

    fn parse_global_checksum(data: &[Data]) -> u16 {
        todo!()
    }
}

impl From<&[Data]> for CartridgeHeader {
    fn from(data: &[Data]) -> Self {
        Self {
            old_licensee_code: Self::parse_old_licensee_code(
                data[(0x014B - CARTRIDGE_HEADER_START_ADDR) as usize],
            ),
            entry_point: Self::parse_entry_point(&data[0x0000..=0x0003]),
            logo: Self::parse_logo(&data[ENTRY_POINT_SIZE..(ENTRY_POINT_SIZE + LOGO_SIZE)]),
            title: Self::parse_title(&data[0x0034..=0x0043]),

            // TODO: Finish implementing the rest of the parsing for the members
            manufacturer_code: Self::parse_manufacturer_code(data),
            cgb_flag: Self::parse_cgb_flag(data[0]),
            new_licensee_code: Self::parse_new_licensee_code(data),
            sgb_flag: Self::parse_sgb_flag(data[0]),
            cartridge_type: Self::parse_cartridge_type(data[0]),
            rom_size: Self::parse_rom_size(data[0]),
            ram_size: Self::parse_ram_size(data[0]),
            destination_code: Self::parse_destination_code(data[0]),
            version_number: Self::parse_version_number(data[0]),
            header_checksum: Self::parse_header_checksum(data[0]),
            global_checksum: Self::parse_global_checksum(data),
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
