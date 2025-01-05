#[cfg(feature = "avian")]
mod controller_avian;
#[cfg(feature = "rapier")]
mod controller_rapier;

pub mod controller {
    #[cfg(feature = "avian")]
    pub use crate::controller_avian::*;
    #[cfg(feature = "rapier")]
    pub use crate::controller_rapier::*;
}
