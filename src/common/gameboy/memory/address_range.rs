use super::types::{Address, Data};
use std::ops::{Range, RangeInclusive};

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
    fn size(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        match self {
            Self::Single { .. } => 1,
            Self::FullInclusive { lower, upper } => (upper - lower) as usize,
            Self::FullExclusive { lower, upper } => (upper - lower) as usize - 2,
            Self::LowerInclusive { lower, upper } => (upper - lower) as usize - 1,
            Self::UpperInclusive { lower, upper } => (upper - lower) as usize - 1,
        }
    }

    fn lower(&self) -> Address {
        match self {
            Self::Single { addr } => *addr,
            Self::FullInclusive { lower, .. } => *lower,
            Self::FullExclusive { lower, .. } => *lower,
            Self::LowerInclusive { lower, .. } => *lower,
            Self::UpperInclusive { lower, .. } => *lower,
        }
    }

    fn upper(&self) -> Address {
        match self {
            Self::Single { addr } => *addr,
            Self::FullInclusive { upper, .. } => *upper,
            Self::FullExclusive { upper, .. } => *upper,
            Self::LowerInclusive { upper, .. } => *upper,
            Self::UpperInclusive { upper, .. } => *upper,
        }
    }

    fn contains(&self, item: Address) -> bool {
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

    fn is_empty(&self) -> bool {
        match self {
            Self::Single { .. } => false,
            Self::FullInclusive { lower, upper } => lower > upper,
            Self::FullExclusive { lower, upper } => lower >= upper,
            Self::LowerInclusive { lower, upper } => lower >= upper,
            Self::UpperInclusive { lower, upper } => lower >= upper,
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
            assert_eq!(addr_range.size(), 7769);

            let addr_range = AddressRange::FullExclusive {
                lower: 0x7AA2,
                upper: 0xF40D,
            };
            assert_eq!(addr_range.size(), 31081);

            let addr_range = AddressRange::LowerInclusive {
                lower: 0x887D,
                upper: 0xAFDD,
            };
            assert_eq!(addr_range.size(), 10079);

            let addr_range = AddressRange::UpperInclusive {
                lower: 0x3391,
                upper: 0xDBFE,
            };
            assert_eq!(addr_range.size(), 43116);
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
    }
}

// --- UNIT TESTS END ---
