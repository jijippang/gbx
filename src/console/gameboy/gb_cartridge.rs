use super::gb_mmu::{MemoryReadError, MemoryWriteError};
use crate::common::gameboy::memory::address_range::AddressRange;
use crate::common::gameboy::memory::types::{Address, Data};
use mbc::Mbc;
use std::{fmt, ops::RangeInclusive};
use tracing::info;

mod mbc;

const MIN_CARTRIDGE_SIZE: usize = 32768;
const CARTRIDGE_HEADER_START_ADDR: Address = 0x0100;
const ENTRY_POINT_SIZE: usize = 0x0103 - 0x0100 + 1;
const LOGO_SIZE: usize = 0x0133 - 0x0104 + 1;
const GLOBAL_CHECKSUM_SIZE: usize = 0x014F - 0x014E + 1;

#[derive(Debug, Default)]
enum CgbFlag {
    // Strict monochrome mode, no color features are supported
    #[default]
    Dmg,

    // Supports color features but also backwards compatible with monochrome mode
    CgbEnhanced,

    // Only supports color features, will not boot on the original DMG
    CgbOnly,
}

impl From<Data> for CgbFlag {
    fn from(data: Data) -> Self {
        match data {
            0x80 => Self::CgbEnhanced,
            0xC0 => Self::CgbOnly,
            _ => Self::Dmg,
        }
    }
}

#[derive(Debug, Default)]
enum NewLicenseeCode {
    #[default]
    None,
    NintendoResearchAndDevelopment1,
    Capcom,
    Ea,
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
    HiTechExpressions,
    Ljn,
    Matchbox,
    Mattel,
    MiltonBradleyCompany,
    TitusInteractive,
    VirginGamesLtd,
    LucasfilmGames,
    OceanSoftware,
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
    ChunsoftCo,
    VideoSystem,
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
            b"08" => Ok(Self::Capcom),
            b"13" => Ok(Self::Ea),
            b"18" => Ok(Self::HudsonSoft),
            b"19" => Ok(Self::BAi),
            b"20" => Ok(Self::Kss),
            b"22" => Ok(Self::PlanningOfficeWada),
            b"24" => Ok(Self::PcmComplete),
            b"25" => Ok(Self::SanX),
            b"28" => Ok(Self::Kemco),
            b"29" => Ok(Self::SetaCorporation),
            b"30" => Ok(Self::Viacom),
            b"31" => Ok(Self::Nintendo),
            b"32" => Ok(Self::Bandai),
            b"33" => Ok(Self::OceanSoftwareAcclaimEntertainment),
            b"34" => Ok(Self::Konami),
            b"35" => Ok(Self::HectorSoft),
            b"37" => Ok(Self::Taito),
            b"38" => Ok(Self::HudsonSoft),
            b"39" => Ok(Self::Banpresto),
            b"41" => Ok(Self::Ubisoft),
            b"42" => Ok(Self::Atlus),
            b"44" => Ok(Self::MalibuInteractive),
            b"46" => Ok(Self::Angel),
            b"47" => Ok(Self::BulletProofSoftware),
            b"49" => Ok(Self::Irem),
            b"50" => Ok(Self::Absolute),
            b"51" => Ok(Self::AcclaimEntertainment),
            b"52" => Ok(Self::Activision),
            b"53" => Ok(Self::SammyUsaCorporation),
            b"54" => Ok(Self::Konami),
            b"55" => Ok(Self::HiTechExpressions),
            b"56" => Ok(Self::Ljn),
            b"57" => Ok(Self::Matchbox),
            b"58" => Ok(Self::Mattel),
            b"59" => Ok(Self::MiltonBradleyCompany),
            b"60" => Ok(Self::TitusInteractive),
            b"61" => Ok(Self::VirginGamesLtd),
            b"64" => Ok(Self::LucasfilmGames),
            b"67" => Ok(Self::OceanSoftware),
            b"69" => Ok(Self::Ea),
            b"70" => Ok(Self::Infogrames),
            b"71" => Ok(Self::InterplayEntertainment),
            b"72" => Ok(Self::Broderbund),
            b"73" => Ok(Self::SculpturedSoftware),
            b"75" => Ok(Self::TheSalesCurveLimited),
            b"78" => Ok(Self::Thq),
            b"79" => Ok(Self::Accolade),
            b"80" => Ok(Self::MisawaEntertainment),
            b"83" => Ok(Self::LozcG),
            b"86" => Ok(Self::TokumaShoten),
            b"87" => Ok(Self::TsukudaOriginal),
            b"91" => Ok(Self::ChunsoftCo),
            b"92" => Ok(Self::VideoSystem),
            b"93" => Ok(Self::OceanSoftwareAcclaimEntertainment),
            b"95" => Ok(Self::Varie),
            b"96" => Ok(Self::Yonezawa),
            b"97" => Ok(Self::Kaneko),
            b"99" => Ok(Self::PackInVideo),
            b"9H" => Ok(Self::BottomUp),
            b"A4" => Ok(Self::KonamiYuGiOh),
            b"BL" => Ok(Self::Mto),
            b"DK" => Ok(Self::Kodansha),
            _ => Err(format!(
                "Unknown data: {:#X?}, unable to deserialize into NewLicenseeCode",
                data
            )),
        }
    }
}

#[derive(Debug, Default)]
enum SgbFlag {
    #[default]
    SgbUnsupported,
    SgbSupported,
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
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    RomRam,
    RomRamBattery,
    Mmm01,
    Mmm01Ram,
    Mmm01RamBattery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
    Mbc6,
    Mbc7SensorRumbleRamBattery,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1RamBattery,
}

impl TryFrom<Data> for CartridgeType {
    type Error = String;

    fn try_from(data: Data) -> Result<Self, Self::Error> {
        match data {
            0x00 => Ok(Self::RomOnly),
            0x01 => Ok(Self::Mbc1),
            0x02 => Ok(Self::Mbc1Ram),
            0x03 => Ok(Self::Mbc1RamBattery),
            0x05 => Ok(Self::Mbc2),
            0x06 => Ok(Self::Mbc2Battery),
            0x08 => Ok(Self::RomRam),
            0x09 => Ok(Self::RomRamBattery),
            0x0B => Ok(Self::Mmm01),
            0x0C => Ok(Self::Mmm01Ram),
            0x0D => Ok(Self::Mmm01RamBattery),
            0x0F => Ok(Self::Mbc3TimerBattery),
            0x10 => Ok(Self::Mbc3TimerRamBattery),
            0x11 => Ok(Self::Mbc3),
            0x12 => Ok(Self::Mbc3Ram),
            0x13 => Ok(Self::Mbc3RamBattery),
            0x19 => Ok(Self::Mbc5),
            0x1A => Ok(Self::Mbc5Ram),
            0x1B => Ok(Self::Mbc5RamBattery),
            0x1C => Ok(Self::Mbc5Rumble),
            0x1D => Ok(Self::Mbc5RumbleRam),
            0x1E => Ok(Self::Mbc5RumbleRamBattery),
            0x20 => Ok(Self::Mbc6),
            0x22 => Ok(Self::Mbc7SensorRumbleRamBattery),
            0xFC => Ok(Self::PocketCamera),
            0xFD => Ok(Self::BandaiTama5),
            0xFE => Ok(Self::HuC3),
            0xFF => Ok(Self::HuC1RamBattery),
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
    Bandai,
    NewLicenseeCode,
    Konami,
    HectorSoft,
    Banpresto,
    EntertainmentInteractive,
    Gremlin,
    Ubisoft,
    Atlus,
    MalibuInteractive,
    Angel,
    SpectrumHoloByte,
    Irem,
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
    OceanSoftware,
    ElectroBrain,
    InterplayEntertainment,
    Broderbund,
    SculpturedSoftware,
    TheSalesCurveLimited,
    Thq,
    Accolade,
    TriffixEntertainment,
    MicroProse,
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
    Arc,
    NihonBussan,
    Tecmo,
    Imagineer,
    Nova,
    HoriElectric,
    Kawada,
    Takara,
    TechnosJapan,
    ToeiAnimation,
    Toho,
    Namco,
    AsciiCorporation,
    SquareEnix,
    HalLaboratory,
    Snk,
    PonyCanyon,
    CultureBrain,
    Sunsoft,
    SonyImagesoft,
    Taito,
    Square,
    DataEast,
    TonkinHouse,
    Koei,
    Ufl,
    UltraGames,
    VapInc,
    UseCorporation,
    Meldac,
    Sofel,
    Quest,
    SigmaEnterprises,
    AskKodanshaCo,
    CopyaSystem,
    Tomy,
    NipponComputerSystems,
    HumanEnt,
    Altron,
    TowaChiki,
    Yutaka,
    Epoch,
    Athena,
    AsmikAceEntertainment,
    Natsume,
    KingRecords,
    SonyRecords,
    Igs,
    AWave,
    ExtremeEntertainment,
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

    fn parse_manufacturer_code(data: &[Data], cgb_flag: &CgbFlag) -> Option<String> {
        match cgb_flag {
            CgbFlag::CgbEnhanced | CgbFlag::CgbOnly => Some(
                String::from_utf8_lossy(data)
                    .trim_end_matches(char::from(0x00))
                    .to_owned(),
            ),
            _ => None,
        }
    }

    fn parse_cgb_flag(data: Data) -> CgbFlag {
        data.into()
    }

    fn parse_new_licensee_code(
        data: &[Data],
        old_licensee_code: &OldLicenseeCode,
    ) -> Option<NewLicenseeCode> {
        match old_licensee_code {
            OldLicenseeCode::NewLicenseeCode => {
                Some(data.try_into().unwrap_or_else(|err| panic!("{}", err)))
            }
            _ => None,
        }
    }

    fn parse_sgb_flag(data: Data) -> SgbFlag {
        data.into()
    }

    fn parse_cartridge_type(data: Data) -> CartridgeType {
        data.try_into().unwrap_or_else(|err| panic!("{}", err))
    }

    fn parse_rom_size(data: Data) -> RomSize {
        data.try_into().unwrap_or_else(|err| panic!("{}", err))
    }

    fn parse_ram_size(data: Data) -> RamSize {
        data.try_into().unwrap_or_else(|err| panic!("{}", err))
    }

    fn parse_destination_code(data: Data) -> DestinationCode {
        data.try_into().unwrap_or_else(|err| panic!("{}", err))
    }

    fn parse_old_licensee_code(data: Data) -> OldLicenseeCode {
        data.try_into().unwrap_or_else(|err| panic!("{}", err))
    }

    fn parse_version_number(data: Data) -> Data {
        data
    }

    fn parse_header_checksum(data: Data) -> Data {
        data
    }

    fn parse_global_checksum(data: &[Data]) -> u16 {
        u16::from_be_bytes(data.try_into().unwrap_or_else(|_| {
            panic!(
                "Data was {} bytes long, global checksum must be {} bytes long",
                data.len(),
                GLOBAL_CHECKSUM_SIZE
            )
        }))
    }
}

impl fmt::Display for CartridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cartridge Header:")?;
        writeln!(f, "entry_point: {:?}", self.entry_point)?;
        writeln!(f, "logo: {:?}", self.logo)?;
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "cgb_flag: {:?}", self.cgb_flag)?;
        writeln!(f, "manufacturer_code: {:?}", self.manufacturer_code)?;
        writeln!(f, "old_licensee_code: {:?}", self.old_licensee_code)?;
        writeln!(f, "new_licensee_code: {:?}", self.new_licensee_code)?;
        writeln!(f, "sgb_flag: {:?}", self.sgb_flag)?;
        writeln!(f, "cartridge_type: {:?}", self.cartridge_type)?;
        writeln!(f, "rom_size: {:?}", self.rom_size)?;
        writeln!(f, "ram_size: {:?}", self.ram_size)?;
        writeln!(f, "destination_code: {:?}", self.destination_code)?;
        writeln!(f, "version_number: {:?}", self.version_number)?;
        writeln!(f, "header_checksum: {:#X?}", self.header_checksum)?;
        write!(f, "global_checksum: {:#X?}", self.global_checksum)
    }
}

impl From<&[Data]> for CartridgeHeader {
    fn from(data: &[Data]) -> Self {
        let entry_point_addr_range: RangeInclusive<usize> = AddressRange::FullInclusive {
            lower: 0x0100,
            upper: 0x0103,
        }
        .checked_sub(CARTRIDGE_HEADER_START_ADDR)
        .expect("Address Range Error")
        .into();

        let logo_addr_range: RangeInclusive<usize> = AddressRange::FullInclusive {
            lower: 0x0104,
            upper: 0x0133,
        }
        .checked_sub(CARTRIDGE_HEADER_START_ADDR)
        .expect("Address Range Error")
        .into();

        let title_addr_range: RangeInclusive<usize> = AddressRange::FullInclusive {
            lower: 0x0134,
            upper: 0x0143,
        }
        .checked_sub(CARTRIDGE_HEADER_START_ADDR)
        .expect("Address Range Error")
        .into();

        let manufacturer_code_addr_range: RangeInclusive<usize> = AddressRange::FullInclusive {
            lower: 0x013F,
            upper: 0x0142,
        }
        .checked_sub(CARTRIDGE_HEADER_START_ADDR)
        .expect("Address Range Error")
        .into();

        let cgb_flag_addr_range: usize = AddressRange::Single { addr: 0x0143 }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let new_licensee_code_addr_range: RangeInclusive<usize> = AddressRange::FullInclusive {
            lower: 0x0144,
            upper: 0x0145,
        }
        .checked_sub(CARTRIDGE_HEADER_START_ADDR)
        .expect("Address Range Error")
        .into();

        let sgb_flag_addr_range: usize = AddressRange::Single { addr: 0x0146 }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let cartridge_type_addr_range: usize = AddressRange::Single { addr: 0x0147 }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let rom_size_addr_range: usize = AddressRange::Single { addr: 0x0148 }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let ram_size_addr_range: usize = AddressRange::Single { addr: 0x0149 }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let destination_code_addr_range: usize = AddressRange::Single { addr: 0x014A }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let old_licensee_code_addr_range: usize = AddressRange::Single { addr: 0x014B }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let version_number_addr_range: usize = AddressRange::Single { addr: 0x014C }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let header_checksum_addr_range: usize = AddressRange::Single { addr: 0x014D }
            .checked_sub(CARTRIDGE_HEADER_START_ADDR)
            .expect("Address Range Error")
            .lower()
            .into();

        let global_checksum_addr_range: RangeInclusive<usize> = AddressRange::FullInclusive {
            lower: 0x014E,
            upper: 0x014F,
        }
        .checked_sub(CARTRIDGE_HEADER_START_ADDR)
        .expect("Address Range Error")
        .into();

        let entry_point = Self::parse_entry_point(&data[entry_point_addr_range]);
        let logo = Self::parse_logo(&data[logo_addr_range]);
        let title = Self::parse_title(&data[title_addr_range]);
        let cgb_flag = Self::parse_cgb_flag(data[cgb_flag_addr_range]);
        let manufacturer_code =
            Self::parse_manufacturer_code(&data[manufacturer_code_addr_range], &cgb_flag);
        let old_licensee_code = Self::parse_old_licensee_code(data[old_licensee_code_addr_range]);
        let new_licensee_code =
            Self::parse_new_licensee_code(&data[new_licensee_code_addr_range], &old_licensee_code);
        let sgb_flag = Self::parse_sgb_flag(data[sgb_flag_addr_range]);
        let cartridge_type = Self::parse_cartridge_type(data[cartridge_type_addr_range]);
        let rom_size = Self::parse_rom_size(data[rom_size_addr_range]);
        let ram_size = Self::parse_ram_size(data[ram_size_addr_range]);
        let destination_code = Self::parse_destination_code(data[destination_code_addr_range]);
        let version_number = Self::parse_version_number(data[version_number_addr_range]);
        let header_checksum = Self::parse_header_checksum(data[header_checksum_addr_range]);
        let global_checksum = Self::parse_global_checksum(&data[global_checksum_addr_range]);

        Self {
            entry_point: entry_point,
            logo: logo,
            title: title,
            manufacturer_code: manufacturer_code,
            cgb_flag: cgb_flag,
            new_licensee_code: new_licensee_code,
            sgb_flag: sgb_flag,
            cartridge_type: cartridge_type,
            rom_size: rom_size,
            ram_size: ram_size,
            destination_code: destination_code,
            old_licensee_code: old_licensee_code,
            version_number: version_number,
            header_checksum: header_checksum,
            global_checksum: global_checksum,
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

        let header = data[0x0100..=0x014F].into();
        info!("{}", header);

        Ok(Self {
            header: header,
            // TODO: Default construct the rest of the struct fields for now
            ..Default::default()
        })
    }
}
