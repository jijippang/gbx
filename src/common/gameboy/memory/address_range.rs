use super::types::Address;
use std::error::Error;
use std::fmt;
use std::ops::{Range, RangeInclusive};

#[derive(Debug, PartialEq)]
pub enum AddressRangeError {
    Underflow(String),
    Overflow(String),
}

impl fmt::Display for AddressRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Underflow(err_str) => write!(f, "{}", err_str),
            Self::Overflow(err_str) => write!(f, "{}", err_str),
        }
    }
}

impl Error for AddressRangeError {}

#[derive(Debug, PartialEq)]
pub enum AddressRange {
    // [a, a]
    Single { addr: Address },

    // [a, b]
    FullInclusive { lower: Address, upper: Address },

    // (a, b)
    FullExclusive { lower: Address, upper: Address },

    // [a, b)
    LowerInclusive { lower: Address, upper: Address },

    // (a, b]
    UpperInclusive { lower: Address, upper: Address },
}

impl AddressRange {
    pub fn size(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        match self {
            Self::Single { .. } => 1,
            Self::FullInclusive { lower, upper } => (upper - lower) as usize + 1,
            Self::FullExclusive { lower, upper } => (upper - lower) as usize - 1,
            Self::LowerInclusive { lower, upper } => (upper - lower) as usize,
            Self::UpperInclusive { lower, upper } => (upper - lower) as usize,
        }
    }

    pub fn lower(&self) -> Address {
        match self {
            Self::Single { addr } => *addr,
            Self::FullInclusive { lower, .. } => *lower,
            Self::FullExclusive { lower, .. } => *lower,
            Self::LowerInclusive { lower, .. } => *lower,
            Self::UpperInclusive { lower, .. } => *lower,
        }
    }

    pub fn upper(&self) -> Address {
        match self {
            Self::Single { addr } => *addr,
            Self::FullInclusive { upper, .. } => *upper,
            Self::FullExclusive { upper, .. } => *upper,
            Self::LowerInclusive { upper, .. } => *upper,
            Self::UpperInclusive { upper, .. } => *upper,
        }
    }

    pub fn contains(&self, item: Address) -> bool {
        if self.is_empty() {
            return false;
        }

        match self {
            Self::Single { addr } => item == *addr,
            Self::FullInclusive { lower, upper } => {
                if item >= *lower && item <= *upper {
                    true
                } else {
                    false
                }
            }
            Self::FullExclusive { lower, upper } => {
                if item > *lower && item < *upper {
                    true
                } else {
                    false
                }
            }
            Self::LowerInclusive { lower, upper } => {
                if item >= *lower && item < *upper {
                    true
                } else {
                    false
                }
            }
            Self::UpperInclusive { lower, upper } => {
                if item > *lower && item <= *upper {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single { .. } => false,
            Self::FullInclusive { lower, upper } => lower > upper,
            Self::FullExclusive { lower, upper } => lower >= upper,
            Self::LowerInclusive { lower, upper } => lower >= upper,
            Self::UpperInclusive { lower, upper } => lower >= upper,
        }
    }

    pub fn checked_add(self, offset_addr: Address) -> Result<Self, AddressRangeError> {
        match self {
            Self::Single { addr } => {
                let new_addr = addr.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base address: {:#X} resulted in an overflow",
                        offset_addr, addr
                    ))
                })?;

                Ok(Self::Single { addr: new_addr })
            }
            Self::FullInclusive { lower, upper } => {
                let new_lower = lower.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base lower address: {:#X} resulted in an overflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base upper address: {:#X} resulted in an overflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::FullInclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
            Self::FullExclusive { lower, upper } => {
                let new_lower = lower.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base lower address: {:#X} resulted in an overflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base upper address: {:#X} resulted in an overflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::FullExclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
            Self::LowerInclusive { lower, upper } => {
                let new_lower = lower.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base lower address: {:#X} resulted in an overflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base upper address: {:#X} resulted in an overflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::LowerInclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
            Self::UpperInclusive { lower, upper } => {
                let new_lower = lower.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base lower address: {:#X} resulted in an overflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_add(offset_addr).ok_or_else(|| {
                    AddressRangeError::Overflow(format!(
                        "Adding offset address: {:#X} to base upper address: {:#X} resulted in an overflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::UpperInclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
        }
    }

    pub fn checked_sub(self, offset_addr: Address) -> Result<Self, AddressRangeError> {
        match self {
            Self::Single { addr } => {
                let new_addr = addr.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base address: {:#X} resulted in an underflow",
                        offset_addr, addr
                    ))
                })?;

                Ok(Self::Single { addr: new_addr })
            }
            Self::FullInclusive { lower, upper } => {
                let new_lower = lower.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base lower address: {:#X} resulted in an underflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base upper address: {:#X} resulted in an underflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::FullInclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
            Self::FullExclusive { lower, upper } => {
                let new_lower = lower.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base lower address: {:#X} resulted in an underflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base upper address: {:#X} resulted in an underflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::FullExclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
            Self::LowerInclusive { lower, upper } => {
                let new_lower = lower.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base lower address: {:#X} resulted in an underflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base upper address: {:#X} resulted in an underflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::LowerInclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
            Self::UpperInclusive { lower, upper } => {
                let new_lower = lower.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base lower address: {:#X} resulted in an underflow",
                        offset_addr, lower
                    ))
                })?;

                let new_upper = upper.checked_sub(offset_addr).ok_or_else(|| {
                    AddressRangeError::Underflow(format!(
                        "Subtracting offset address: {:#X} from base upper address: {:#X} resulted in an underflow",
                        offset_addr, upper
                    ))
                })?;

                Ok(Self::UpperInclusive {
                    lower: new_lower,
                    upper: new_upper,
                })
            }
        }
    }
}

impl From<Address> for AddressRange {
    fn from(addr: Address) -> Self {
        Self::Single { addr: addr }
    }
}

impl From<Range<Address>> for AddressRange {
    fn from(range: Range<Address>) -> Self {
        Self::LowerInclusive {
            lower: range.start,
            upper: range.end,
        }
    }
}

impl From<RangeInclusive<Address>> for AddressRange {
    fn from(range_inclusive: RangeInclusive<Address>) -> Self {
        Self::FullInclusive {
            lower: *range_inclusive.start(),
            upper: *range_inclusive.end(),
        }
    }
}

impl From<AddressRange> for Range<usize> {
    fn from(addr_range: AddressRange) -> Self {
        match addr_range {
            AddressRange::Single { addr } => addr as usize..(addr + 1) as usize,
            AddressRange::FullInclusive { lower, upper } => lower as usize..(upper + 1) as usize,
            AddressRange::FullExclusive { lower, upper } => (lower + 1) as usize..upper as usize,
            AddressRange::LowerInclusive { lower, upper } => lower as usize..upper as usize,
            AddressRange::UpperInclusive { lower, upper } => {
                (lower + 1) as usize..(upper + 1) as usize
            }
        }
    }
}

impl From<AddressRange> for RangeInclusive<usize> {
    fn from(addr_range: AddressRange) -> Self {
        match addr_range {
            AddressRange::Single { addr } => addr as usize..=addr as usize,
            AddressRange::FullInclusive { lower, upper } => lower as usize..=upper as usize,
            AddressRange::FullExclusive { lower, upper } => {
                (lower + 1) as usize..=(upper - 1) as usize
            }
            AddressRange::LowerInclusive { lower, upper } => lower as usize..=(upper - 1) as usize,
            AddressRange::UpperInclusive { lower, upper } => (lower + 1) as usize..=upper as usize,
        }
    }
}

// --- UNIT TESTS BEGIN ---

#[cfg(test)]
mod tests {
    use super::*;

    mod address_range_tests {
        use super::*;

        #[test]
        fn test_size() {
            let addr_range = AddressRange::Single { addr: 0xFFFF };
            assert_eq!(addr_range.size(), 1);

            let addr_range = AddressRange::FullInclusive {
                lower: 0x2401,
                upper: 0x425A,
            };
            assert_eq!(addr_range.size(), 7770);

            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0xF40D,
            };
            assert_eq!(addr_range.size(), 31082);

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0xAFDD,
            };
            assert_eq!(addr_range.size(), 10080);

            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0xDBFE,
            };
            assert_eq!(addr_range.size(), 43117);
        }

        #[test]
        fn test_lower() {
            let addr_range = AddressRange::Single { addr: 0xFFFF };
            assert_eq!(addr_range.lower(), 0xFFFF);

            let addr_range = AddressRange::FullInclusive {
                lower: 0x2401,
                upper: 0x425A,
            };
            assert_eq!(addr_range.lower(), 0x2401);

            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0xF40D,
            };
            assert_eq!(addr_range.lower(), 0x7AA2);

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0xAFDD,
            };
            assert_eq!(addr_range.lower(), 0x887D);

            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0xDBFE,
            };
            assert_eq!(addr_range.lower(), 0x3391);
        }

        #[test]
        fn test_upper() {
            let addr_range = AddressRange::Single { addr: 0xFFFF };
            assert_eq!(addr_range.upper(), 0xFFFF);

            let addr_range = AddressRange::FullInclusive {
                lower: 0x2401,
                upper: 0x425A,
            };
            assert_eq!(addr_range.upper(), 0x425A);

            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0xF40D,
            };
            assert_eq!(addr_range.upper(), 0xF40D);

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0xAFDD,
            };
            assert_eq!(addr_range.upper(), 0xAFDD);

            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0xDBFE,
            };
            assert_eq!(addr_range.upper(), 0xDBFE);
        }

        #[test]
        fn test_contains() {
            let addr_range = AddressRange::Single { addr: 0xFFFF };
            let item = 0xFFFF;
            assert!(addr_range.contains(item));
            let item = 0x420A;
            assert!(!addr_range.contains(item));

            let addr_range = AddressRange::FullInclusive {
                lower: 0x2401,
                upper: 0x9412,
            };
            let item = 0x842A;
            assert!(addr_range.contains(item));
            let item = 0xD990;
            assert!(!addr_range.contains(item));

            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0x9984,
            };
            let item = 0x7FF2;
            assert!(addr_range.contains(item));
            let item = 0xE82A;
            assert!(!addr_range.contains(item));

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0xAFDD,
            };
            let item = 0x99DA;
            assert!(addr_range.contains(item));
            let item = 0x40EE;
            assert!(!addr_range.contains(item));

            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0xFFA9,
            };
            let item = 0xA09E;
            assert!(addr_range.contains(item));
            let item = 0x0876;
            assert!(!addr_range.contains(item));
        }

        #[test]
        fn test_is_empty() {
            let addr_range = AddressRange::Single { addr: 0xFFFF };
            assert!(!addr_range.is_empty());

            let addr_range = AddressRange::FullInclusive {
                lower: 0x2401,
                upper: 0x9412,
            };
            assert!(!addr_range.is_empty());
            let addr_range = AddressRange::FullInclusive {
                lower: 0x2401,
                upper: 0x2401,
            };
            assert!(!addr_range.is_empty());
            let addr_range = AddressRange::FullInclusive {
                lower: 0x2401,
                upper: 0x10FA,
            };
            assert!(addr_range.is_empty());

            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0x9984,
            };
            assert!(!addr_range.is_empty());
            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0x7AA2,
            };
            assert!(addr_range.is_empty());
            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0x3001,
            };
            assert!(addr_range.is_empty());

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0xAFDD,
            };
            assert!(!addr_range.is_empty());
            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0x887D,
            };
            assert!(addr_range.is_empty());
            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0x529F,
            };
            assert!(addr_range.is_empty());

            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0xFFA9,
            };
            assert!(!addr_range.is_empty());
            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0x3391,
            };
            assert!(addr_range.is_empty());
            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0x004F,
            };
            assert!(addr_range.is_empty());
        }

        #[test]
        fn test_checked_add() {
            let addr_range = AddressRange::Single { addr: Address::MAX }.checked_add(0x0002);
            assert!(matches!(addr_range, Err(AddressRangeError::Overflow(_))));
            let addr_range = AddressRange::Single { addr: 0x0420 }.checked_add(0x0002);
            assert_eq!(addr_range, Ok(AddressRange::Single { addr: 0x0422 }));

            let addr_range = AddressRange::FullInclusive {
                lower: Address::MAX,
                upper: 0x9412,
            }
            .checked_add(0x0003);
            assert!(matches!(addr_range, Err(AddressRangeError::Overflow(_))));
            let addr_range = AddressRange::FullInclusive {
                lower: 0x0245,
                upper: 0x9412,
            }
            .checked_add(0x0003);
            assert_eq!(
                addr_range,
                Ok(AddressRange::FullInclusive {
                    lower: 0x0248,
                    upper: 0x9415,
                })
            );

            let addr_range = AddressRange::FullExclusive {
                lower: Address::MAX,
                upper: 0x9412,
            }
            .checked_add(0x0001);
            assert!(matches!(addr_range, Err(AddressRangeError::Overflow(_))));
            let addr_range = AddressRange::FullExclusive {
                lower: 0x0245,
                upper: 0x9412,
            }
            .checked_add(0x0001);
            assert_eq!(
                addr_range,
                Ok(AddressRange::FullExclusive {
                    lower: 0x0246,
                    upper: 0x9413,
                })
            );

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x40FA,
                upper: Address::MAX,
            }
            .checked_add(0x0004);
            assert!(matches!(addr_range, Err(AddressRangeError::Overflow(_))));
            let addr_range = AddressRange::LowerInclusive {
                lower: 0x0245,
                upper: 0x9412,
            }
            .checked_add(0x0004);
            assert_eq!(
                addr_range,
                Ok(AddressRange::LowerInclusive {
                    lower: 0x0249,
                    upper: 0x9416,
                })
            );

            let addr_range = AddressRange::UpperInclusive {
                lower: Address::MAX,
                upper: Address::MAX,
            }
            .checked_add(0x0003);
            assert!(matches!(addr_range, Err(AddressRangeError::Overflow(_))));
            let addr_range = AddressRange::UpperInclusive {
                lower: 0x0245,
                upper: 0x9412,
            }
            .checked_add(0x0003);
            assert_eq!(
                addr_range,
                Ok(AddressRange::UpperInclusive {
                    lower: 0x0248,
                    upper: 0x9415,
                })
            );
        }

        #[test]
        fn test_checked_sub() {
            let addr_range = AddressRange::Single { addr: Address::MIN }.checked_sub(0x0002);
            assert!(matches!(addr_range, Err(AddressRangeError::Underflow(_))));
            let addr_range = AddressRange::Single { addr: 0x0420 }.checked_sub(0x0002);
            assert_eq!(addr_range, Ok(AddressRange::Single { addr: 0x041E }));

            let addr_range = AddressRange::FullInclusive {
                lower: Address::MIN,
                upper: 0x9412,
            }
            .checked_sub(0x0003);
            assert!(matches!(addr_range, Err(AddressRangeError::Underflow(_))));
            let addr_range = AddressRange::FullInclusive {
                lower: 0x0245,
                upper: 0x9414,
            }
            .checked_sub(0x0003);
            assert_eq!(
                addr_range,
                Ok(AddressRange::FullInclusive {
                    lower: 0x0242,
                    upper: 0x9411,
                })
            );

            let addr_range = AddressRange::FullExclusive {
                lower: Address::MIN,
                upper: 0x9412,
            }
            .checked_sub(0x0001);
            assert!(matches!(addr_range, Err(AddressRangeError::Underflow(_))));
            let addr_range = AddressRange::FullExclusive {
                lower: 0x0245,
                upper: 0x9412,
            }
            .checked_sub(0x0001);
            assert_eq!(
                addr_range,
                Ok(AddressRange::FullExclusive {
                    lower: 0x0244,
                    upper: 0x9411,
                })
            );

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x40FA,
                upper: Address::MIN,
            }
            .checked_sub(0x0004);
            assert!(matches!(addr_range, Err(AddressRangeError::Underflow(_))));
            let addr_range = AddressRange::LowerInclusive {
                lower: 0x0245,
                upper: 0x9417,
            }
            .checked_sub(0x0004);
            assert_eq!(
                addr_range,
                Ok(AddressRange::LowerInclusive {
                    lower: 0x0241,
                    upper: 0x9413,
                })
            );

            let addr_range = AddressRange::UpperInclusive {
                lower: Address::MIN,
                upper: Address::MIN,
            }
            .checked_sub(0x0003);
            assert!(matches!(addr_range, Err(AddressRangeError::Underflow(_))));
            let addr_range = AddressRange::UpperInclusive {
                lower: 0x0245,
                upper: 0x9419,
            }
            .checked_sub(0x0003);
            assert_eq!(
                addr_range,
                Ok(AddressRange::UpperInclusive {
                    lower: 0x0242,
                    upper: 0x9416,
                })
            );
        }
    }
}

// --- UNIT TESTS END ---
