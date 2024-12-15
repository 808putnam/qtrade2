// qtrade:
// Had to switch to new implementation for Discriminator through-out account_load.rs.
// References to discriminator() and discriminator.len were replaced below.
// For reference, here is the new implementation:
/// Unique identifier for a type.
///
/// This is not a trait you should derive manually, as various Anchor macros already derive it
/// internally.
///
/// Prior to Anchor v0.31, discriminators were always 8 bytes in size. However, starting with Anchor
/// v0.31, it is possible to override the default discriminators, and discriminator length is no
/// longer fixed, which means this trait can also be implemented for non-Anchor programs.
///
/// It's important that the discriminator is always unique for the type you're implementing it
/// for. While the discriminator can be at any length (including zero), the IDL generation does not
/// currently allow empty discriminators for safety and convenience reasons. However, the trait
/// definition still allows empty discriminators because some non-Anchor programs, e.g. the SPL
/// Token program, don't have account discriminators. In that case, safety checks should never
/// depend on the discriminator.
// pub trait Discriminator {
//     /// Discriminator slice.
//     ///
//     /// See [`Discriminator`] trait documentation for more information.
//     const DISCRIMINATOR: &'static [u8];
// }

use anchor_lang::error::{Error, ErrorCode};
use anchor_lang::{Key, Owner, Result, ToAccountInfos, ZeroCopy};
use arrayref::array_ref;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem;
use std::ops::DerefMut;

#[derive(Clone)]
pub struct AccountLoad<'info, T: ZeroCopy + Owner> {
    acc_info: AccountInfo<'info>,
    phantom: PhantomData<&'info T>,
}

impl<'info, T: ZeroCopy + Owner> AccountLoad<'info, T> {
    fn new(acc_info: AccountInfo<'info>) -> AccountLoad<'info, T> {
        Self {
            acc_info,
            phantom: PhantomData,
        }
    }

    /// Constructs a new `Loader` from a previously initialized account.
    #[inline(never)]
    pub fn try_from(acc_info: &AccountInfo<'info>) -> Result<AccountLoad<'info, T>> {
        if acc_info.owner != &T::owner() {
            return Err(Error::from(ErrorCode::AccountOwnedByWrongProgram)
                .with_pubkeys((*acc_info.owner, T::owner())));
        }
        let data: &[u8] = &acc_info.try_borrow_data()?;
        if data.len() < T::DISCRIMINATOR.len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }
        // Discriminator must match.
        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::DISCRIMINATOR {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(AccountLoad::new(acc_info.clone()))
    }

    /// Constructs a new `Loader` from an uninitialized account.
    #[inline(never)]
    pub fn try_from_unchecked(
        _program_id: &Pubkey,
        acc_info: &AccountInfo<'info>,
    ) -> Result<AccountLoad<'info, T>> {
        if acc_info.owner != &T::owner() {
            return Err(Error::from(ErrorCode::AccountOwnedByWrongProgram)
                .with_pubkeys((*acc_info.owner, T::owner())));
        }
        Ok(AccountLoad::new(acc_info.clone()))
    }

    /// Returns a `RefMut` to the account data structure for reading or writing.
    /// Should only be called once, when the account is being initialized.
    pub fn load_init(&self) -> Result<RefMut<T>> {
        // AccountInfo api allows you to borrow mut even if the account isn't
        // writable, so add this check for a better dev experience.
        if !self.acc_info.is_writable {
            return Err(ErrorCode::AccountNotMutable.into());
        }

        let mut data = self.acc_info.try_borrow_mut_data()?;

        // The discriminator should be zero, since we're initializing.
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        let discriminator = u64::from_le_bytes(disc_bytes);
        if discriminator != 0 {
            return Err(ErrorCode::AccountDiscriminatorAlreadySet.into());
        }

        // write discriminator
        data[..8].copy_from_slice(&T::DISCRIMINATOR);

        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(&mut data.deref_mut()[8..mem::size_of::<T>() + 8])
        }))
    }

    /// Returns a `RefMut` to the account data structure for reading or writing directly.
    /// There is no need to convert AccountInfo to AccountLoad.
    /// So it is necessary to check the owner
    pub fn load_data_mut<'a>(acc_info: &'a AccountInfo) -> Result<RefMut<'a, T>> {
        if acc_info.owner != &T::owner() {
            return Err(Error::from(ErrorCode::AccountOwnedByWrongProgram)
                .with_pubkeys((*acc_info.owner, T::owner())));
        }
        if !acc_info.is_writable {
            return Err(ErrorCode::AccountNotMutable.into());
        }

        let data = acc_info.try_borrow_mut_data()?;
        if data.len() < T::DISCRIMINATOR.len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::DISCRIMINATOR {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(&mut data.deref_mut()[8..mem::size_of::<T>() + 8])
        }))
    }

    /// Returns a Ref to the account data structure for reading.
    pub fn load(&self) -> Result<Ref<T>> {
        let data = self.acc_info.try_borrow_data()?;
        if data.len() < T::DISCRIMINATOR.len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::DISCRIMINATOR {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(Ref::map(data, |data| {
            bytemuck::from_bytes(&data[8..mem::size_of::<T>() + 8])
        }))
    }

    /// Returns a `RefMut` to the account data structure for reading or writing.
    pub fn load_mut(&self) -> Result<RefMut<T>> {
        // AccountInfo api allows you to borrow mut even if the account isn't
        // writable, so add this check for a better dev experience.
        if !self.acc_info.is_writable {
            return Err(ErrorCode::AccountNotMutable.into());
        }

        let data = self.acc_info.try_borrow_mut_data()?;
        if data.len() < T::DISCRIMINATOR.len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::DISCRIMINATOR {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(&mut data.deref_mut()[8..mem::size_of::<T>() + 8])
        }))
    }
}

impl<'info, T: ZeroCopy + Owner> AsRef<AccountInfo<'info>> for AccountLoad<'info, T> {
    fn as_ref(&self) -> &AccountInfo<'info> {
        &self.acc_info
    }
}
impl<'info, T: ZeroCopy + Owner> ToAccountInfos<'info> for AccountLoad<'info, T> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.acc_info.clone()]
    }
}

impl<'info, T: ZeroCopy + Owner> Key for AccountLoad<'info, T> {
    fn key(&self) -> Pubkey {
        *self.acc_info.key
    }
}
