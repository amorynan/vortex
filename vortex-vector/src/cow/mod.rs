// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

mod bit_buffer;
mod buffer;
mod mask;

use std::{fmt, mem};

/// A trait defining an immutable type that can be converted into a mutable counterpart.
///
/// The [`Clone`] implementation should be a cheap copy that only increments reference counts.
pub trait IntoMut: Clone {
    /// The mutable form of the type that it can be converted into.
    type Mutable: IntoFrozen<Frozen = Self>;

    /// Converts an immutable value into its mutable counterpart, usually via cloning.
    fn into_mut(self) -> Self::Mutable;
}

/// A trait defining a mutable type that can be converted into an immutable / frozen counterpart.
///
/// The [`Clone`] implementation likely needs to perform a "deep" clone by copying all data to a new
/// allocation.
pub trait IntoFrozen: Clone {
    /// The immutable / frozen form of the type that it can be converted into.
    type Frozen: IntoMut<Mutable = Self>;

    /// Converts a mutable value into its immutable counterpart.
    fn freeze(self) -> Self::Frozen;
}

/// A clone-on-write enum that can hold either an owned immutable or owned mutable value.
#[derive(Clone)]
pub enum Cow<F>
where
    F: IntoMut,
{
    /// A frozen immutable value that is owned.
    Frozen(F),
    /// A mutable value that is owned.
    Mutable(<F as IntoMut>::Mutable),
}

impl<F> Cow<F>
where
    F: IntoMut + Clone,
{
    /// Returns `true` if the data is frozen (if `to_mut` may not be cheap).
    pub fn is_frozen(&self) -> bool {
        matches!(self, Cow::Frozen(_))
    }

    /// Returns `true` if the data is mutable (if `to_mut` would be a no-op).
    pub fn is_mutable(&self) -> bool {
        matches!(self, Cow::Mutable(_))
    }

    /// Extracts the frozen data.
    ///
    /// This will freeze the data if it was not yet frozen.
    pub fn into_frozen(self) -> F {
        match self {
            Cow::Frozen(frozen) => frozen,
            Cow::Mutable(mutable) => mutable.freeze(),
        }
    }

    /// Extracts the mutable data.
    ///
    /// Performs a deep clone of the data if it is not already mutable.
    pub fn into_mut(self) -> <F as IntoMut>::Mutable {
        match self {
            Cow::Frozen(frozen) => frozen.into_mut(),
            Cow::Mutable(mutable) => mutable,
        }
    }

    /// Acquires a reference to the frozen form of the data, if it is frozen.
    pub fn as_frozen(&self) -> Option<&F> {
        match self {
            Cow::Frozen(frozen) => Some(frozen),
            Cow::Mutable(_mutable) => None,
        }
    }

    /// Acquires a reference to the mutable form of the data, if it is mutable.
    pub fn as_mutable(&mut self) -> Option<&mut F::Mutable> {
        match self {
            Cow::Frozen(_frozen) => None,
            Cow::Mutable(mutable) => Some(mutable),
        }
    }
}

impl<F> Cow<F>
where
    F: IntoMut,
    <F as IntoMut>::Mutable: Default,
{
    /// Acquires a reference to the frozen form of the data.
    ///
    /// This will freeze the data if it was not yet frozen.
    ///
    /// We return a mutable reference for flexibility, but since `F` should be an immutable data
    /// type, it should always be reborrowed as an immutable reference.
    #[inline]
    pub fn ensure_frozen(&mut self) -> &F {
        match self {
            Cow::Frozen(frozen) => frozen,
            Cow::Mutable(mutable) => {
                *self = Cow::Frozen(mem::take(mutable).freeze());
                match self {
                    Cow::Frozen(frozen) => frozen,
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }
            }
        }
    }
}

impl<F> Cow<F>
where
    F: IntoMut + Default,
{
    /// Acquires a mutable reference to the mutable form of the data.
    ///
    /// Performs a deep clone of the data if it is not already mutable.
    #[inline]
    pub fn ensure_mut(&mut self) -> &mut <F as IntoMut>::Mutable {
        match self {
            Cow::Frozen(frozen) => {
                *self = Cow::Mutable(mem::take(frozen).into_mut());
                match self {
                    Cow::Mutable(mutable) => mutable,
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }
            }
            Cow::Mutable(mutable) => mutable,
        }
    }
}

impl<F: IntoMut> From<F> for Cow<F> {
    fn from(value: F) -> Self {
        Cow::Frozen(value)
    }
}

impl<F> Default for Cow<F>
where
    F: IntoMut + Default,
{
    fn default() -> Self {
        Cow::Frozen(F::default())
    }
}

impl<F> fmt::Debug for Cow<F>
where
    F: IntoMut + fmt::Debug,
    <F as IntoMut>::Mutable: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cow::Frozen(frozen) => fmt::Debug::fmt(frozen, f),
            Cow::Mutable(mutable) => fmt::Debug::fmt(mutable, f),
        }
    }
}
