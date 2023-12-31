#[derive(Debug, PartialEq)]
pub enum Error {
    UnsupportedType(u16),
    UnsupportedClass(u16),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnsupportedType(value) => {
                write!(f, "Unsupported type {}", value)
            }
            Error::UnsupportedClass(value) => {
                write!(f, "Unsupported class {}", value)
            }
        }
    }
}

/// TYPE  | value and meaning
/// ------+-----------------------------------------
/// A     | a host address
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum Type {
    A = 1,
}

impl TryFrom<u16> for Type {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Type::A),
            _ => Err(Error::UnsupportedType(value)),
        }
    }
}

impl Into<u16> for Type {
    fn into(self) -> u16 {
        self as u16
    }
}

impl Clone for Type {
    fn clone(&self) -> Self {
        match self {
            Type::A => Type::A,
        }
    }
}

/// CLASS  | value and meaning
/// -------+-----------------------------------------
/// IN     | an Internet host
#[repr(u16)]
#[derive(Debug, PartialEq)]
pub enum Class {
    In = 1,
}

impl TryFrom<u16> for Class {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Class::In),
            _ => Err(Error::UnsupportedClass(value)),
        }
    }
}

impl Clone for Class {
    fn clone(&self) -> Self {
        match self {
            Class::In => Class::In,
        }
    }
}

impl Into<u16> for Class {
    fn into(self) -> u16 {
        self as u16
    }
}
