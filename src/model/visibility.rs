#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Public,
    PackagePrivate,
    Protected,
    Private,
}

impl Visibility {
    pub fn from_access_flags(flags: u16) -> Self {
        if flags & 0x0001 != 0 {
            Self::Public
        } else if flags & 0x0002 != 0 {
            Self::Private
        } else if flags & 0x0004 != 0 {
            Self::Protected
        } else {
            Self::PackagePrivate
        }
    }
}