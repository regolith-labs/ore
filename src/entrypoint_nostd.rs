extern crate alloc;
use alloc::rc::Rc;

use core::{cell::RefCell, marker::PhantomData, mem::size_of, ptr::NonNull, slice::from_raw_parts};

use arrayvec::ArrayVec;
use bytemuck::{Pod, Zeroable};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE, NON_DUP_MARKER},
    log,
    pubkey::Pubkey,
};

#[macro_export]
macro_rules! entrypoint_nostd {
    ($process_instruction:ident, $accounts:literal) => {
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            let (program_id, accounts, instruction_data) =
                unsafe { $crate::deserialize_nostd::<$accounts>(input) };
            match $process_instruction(&program_id, &accounts, &instruction_data) {
                Ok(()) => solana_program::entrypoint::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

#[macro_export]
macro_rules! entrypoint_nostd_no_duplicates {
    ($process_instruction:ident, $accounts:literal) => {
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            let Some((program_id, accounts, instruction_data)) =
                                $crate::deserialize_nostd_no_dup::<$accounts>(input)
                            else {
                                // TODO: better error
                                solana_program::log::sol_log("a duplicate account was found");
                                return u64::MAX;
                            };
            // solana_program::entrypoint::SUCCESS
            match $process_instruction(&program_id, &accounts, &instruction_data) {
                Ok(()) => solana_program::entrypoint::SUCCESS,
                Err(error) => error.into(),
            }
        }
    };
}

pub unsafe fn deserialize_nostd<'a, const MAX_ACCOUNTS: usize>(
    input: *mut u8,
) -> (
    &'a Pubkey,
    ArrayVec<NoStdAccountInfo4, MAX_ACCOUNTS>,
    &'a [u8],
) {
    let mut offset: usize = 0;

    // Number of accounts present
    #[allow(clippy::cast_ptr_alignment)]
    let num_accounts = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    // Account Infos
    let mut accounts = ArrayVec::new();
    for _ in 0..num_accounts {
        let dup_info = *(input.add(offset) as *const u8);
        if dup_info == NON_DUP_MARKER {
            // MAGNETAR FIELDS: safety depends on alignment, size
            // 1) we will always be 8 byte aligned due to align_offset
            // 2) solana vm serialization format is consistent so size is ok
            let account_info: &mut NoStdAccountInfo4Inner =
                core::mem::transmute::<&mut u8, _>(&mut *(input.add(offset)));
            // bytemuck::try_from_bytes_mut(from_raw_parts_mut(input.add(offset), 88)).unwrap();

            offset += size_of::<NoStdAccountInfo4Inner>();
            offset += account_info.data_len;
            offset += MAX_PERMITTED_DATA_INCREASE;
            offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
            offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch

            // MAGNETAR FIELDS: reset borrow state right before pushing
            account_info.borrow_state = 0b_0000_0000;
            if accounts
                .try_push(NoStdAccountInfo4 {
                    inner: account_info,
                })
                .is_err()
            {
                log::sol_log("ArrayVec is full. Truncating input accounts");
            };
        } else {
            offset += 8;

            // Duplicate account, clone the original
            if accounts
                .try_push(accounts[dup_info as usize].clone())
                .is_err()
            {
                log::sol_log("ArrayVec is full. Truncating input accounts");
            };
        }
    }

    // Instruction data
    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let instruction_data = { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // Program Id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    (program_id, accounts, instruction_data)
}

pub unsafe fn deserialize_nostd_no_dup<'a, const MAX_ACCOUNTS: usize>(
    input: *mut u8,
) -> Option<(
    &'a Pubkey,
    ArrayVec<NoStdAccountInfo4, MAX_ACCOUNTS>,
    &'a [u8],
)> {
    let mut offset: usize = 0;

    // Number of accounts present
    #[allow(clippy::cast_ptr_alignment)]
    let num_accounts = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    // Account Infos
    let mut accounts = ArrayVec::new();
    for _ in 0..num_accounts {
        let dup_info = *(input.add(offset) as *const u8);
        if dup_info == NON_DUP_MARKER {
            // MAGNETAR FIELDS: safety depends on alignment, size
            // 1) we will always be 8 byte aligned due to align_offset
            // 2) solana vm serialization format is consistent so size is ok
            let account_info: &mut NoStdAccountInfo4Inner =
                core::mem::transmute::<&mut u8, _>(&mut *(input.add(offset)));
            // bytemuck::try_from_bytes_mut(from_raw_parts_mut(input.add(offset), 88)).unwrap();
            offset += size_of::<NoStdAccountInfo4Inner>();
            offset += account_info.data_len;
            offset += MAX_PERMITTED_DATA_INCREASE;
            offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
            offset += size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch

            // MAGNETAR FIELDS: reset borrow state right before pushing
            account_info.borrow_state = 0b_0000_0000;
            if accounts
                .try_push(NoStdAccountInfo4 {
                    inner: account_info,
                })
                .is_err()
            {
                log::sol_log("ArrayVec is full. Truncating input accounts");
            };
        } else {
            return None;
        }
    }

    // Instruction data
    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len = *(input.add(offset) as *const u64) as usize;
    offset += size_of::<u64>();

    let instruction_data = { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // Program Id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    Some((program_id, accounts, instruction_data))
}

#[derive(Clone, PartialEq, Eq)]
#[repr(C)]
pub struct NoStdAccountInfo4 {
    inner: *mut NoStdAccountInfo4Inner,
}

impl NoStdAccountInfo4 {
    /// SAFETY: you must ensure that this pointer IS + REMAINS valid.
    pub unsafe fn from(inner: *mut NoStdAccountInfo4Inner) -> NoStdAccountInfo4 {
        NoStdAccountInfo4 { inner }
    }
}

#[derive(Clone, Pod, Zeroable, Copy, Default)]
#[repr(C)]
pub struct NoStdAccountInfo4Inner {
    /// 0) We reuse the duplicate flag for this. We set it to 0b0000_0000.
    /// 1) We use the first four bits to track state of lamport borrow
    /// 2) We use the second four bits to track state of data borrow
    ///
    /// 4 bit state: [1 bit mutable borrow flag | u3 immmutable borrow flag]
    /// This gives us up to 7 immutable borrows. Note that does not mean 7
    /// duplicate account infos, but rather 7 calls to borrow lamports or
    /// borrow data across all duplicate account infos.
    borrow_state: u8,

    /// Was the transaction signed by this account's public key?
    is_signer: u8,

    /// Is the account writable?
    is_writable: u8,

    /// This account's data contains a loaded program (and is now read-only)
    executable: u8,

    padding: u32,

    /// Public key of the account
    key: Pubkey,
    /// Program that owns this account
    owner: Pubkey,

    /// The lamports in the account.  Modifiable by programs.
    lamports: u64,
    data_len: usize,
}

#[repr(C)]
pub struct AccountMetaC {
    pubkey: *const Pubkey,
    is_writable: bool,
    is_signer: bool,
}

pub struct AccountInfoC {
    pub key: *const Pubkey,   /* Public key of the account */
    pub lamports: *const u64, /* Number of lamports owned by this account */
    pub data_len: u64,        /* Length of data in bytes */
    pub data: *const u8,      /* On-chain data within this account */
    pub owner: *const Pubkey, /* Program that owns this account */
    pub rent_epoch: u64,      /* The epoch at which this account will next owe rent */
    pub is_signer: bool,      /* Transaction was signed by this account's key? */
    pub is_writable: bool,    /* Is the account writable? */
    pub executable: bool, /* This account's data contains a loaded program (and is now read-only) */
}

impl AccountInfoC {
    #[inline(always)]
    pub fn to_meta_c(&self) -> AccountMetaC {
        AccountMetaC {
            pubkey: self.key,
            is_writable: self.is_writable,
            is_signer: self.is_signer,
        }
    }
    #[inline(always)]
    pub fn to_meta_c_signer(&self) -> AccountMetaC {
        AccountMetaC {
            pubkey: self.key,
            is_writable: self.is_writable,
            is_signer: true,
        }
    }
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct InstructionC {
    pub program_id: *const Pubkey,
    pub accounts: *const AccountMetaC,
    pub accounts_len: u64,
    pub data: *const u8,
    pub data_len: u64,
}

pub struct Ref<'a, T: ?Sized> {
    value: &'a T,
    state: NonNull<u8>,
    is_lamport: bool,
}

impl<'a, T: ?Sized> core::ops::Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for Ref<'a, T> {
    // We just need to decrement the immutable borrow count
    fn drop(&mut self) {
        if self.is_lamport {
            unsafe { *self.state.as_mut() -= 1 << 4 };
        } else {
            unsafe { *self.state.as_mut() -= 1 };
        }
    }
}

pub struct RefMut<'a, T: ?Sized> {
    value: &'a mut T,
    state: NonNull<u8>,
    is_lamport: bool,
}

impl<'a, T: ?Sized> core::ops::Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<'a, T: ?Sized> core::ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut <Self as core::ops::Deref>::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for RefMut<'a, T> {
    // We need to unset the mut borrow flag
    fn drop(&mut self) {
        if self.is_lamport {
            unsafe { *self.state.as_mut() &= 0b_0111_1111 };
        } else {
            unsafe { *self.state.as_mut() &= 0b_1111_0111 };
        }
    }
}

/// SAFETY:
/// Within the standard library, RcBox uses repr(C) which guarantees
/// we will always have the layout
///
/// strong: isize,
/// weak: isize,
/// value: T
///
/// For us, T -> RefCell<T>. Since RefCell<T> has T: ?Sized, this
/// guarantees that the inner fields of RefCell are not reordered.
/// So, in conclusion, this type has a stable memory layout.
#[repr(C, align(8))]
pub struct RcRefCellInner<'a, T> {
    strong: isize,
    weak: isize,
    refcell: RefCell<T>,
    phantom_data: PhantomData<&'a mut ()>,
}

impl<'a, T> RcRefCellInner<'a, T> {
    pub fn new(value: T) -> RcRefCellInner<'a, T> {
        RcRefCellInner {
            strong: 2,
            weak: 2,
            refcell: RefCell::new(value),
            phantom_data: PhantomData,
        }
    }

    /// NOTE: when the last Rc is dropped, the strong count will reach
    /// one. So, it will not deallocate, which is fine because the
    /// Rc points to stack memory.
    ///
    /// SAFETY: [RcRefCellInner] must NOT be dropped before this Rc is
    /// used. There can be no safe abstraction that guarantees users
    /// do this because we cannot make Rc inherit the borrowed
    /// lifetime.
    unsafe fn as_rcrc(&self) -> Rc<RefCell<T>> {
        // Rc::from_raw expects pointer to T
        unsafe { Rc::from_raw(&self.refcell as *const RefCell<T>) }
    }
}

#[inline(always)]
const fn offset<T, U>(ptr: *const T, offset: usize) -> *const U {
    unsafe { (ptr as *const u8).add(offset) as *const U }
}

impl NoStdAccountInfo4 {
    pub fn to_info_c(&self) -> AccountInfoC {
        AccountInfoC {
            key: offset(self.inner, 8),
            lamports: offset(self.inner, 72),
            data_len: self.data_len() as u64,
            data: offset(self.inner, 88),
            owner: offset(self.inner, 40),
            rent_epoch: 0,
            is_signer: self.is_signer(),
            is_writable: self.is_writable(),
            executable: self.executable(),
        }
    }
    pub fn to_meta_c(&self) -> AccountMetaC {
        AccountMetaC {
            pubkey: offset(self.inner, 8),
            is_writable: self.is_writable(),
            is_signer: self.is_signer(),
        }
    }

    pub unsafe fn unchecked_info_prep<'a>(
        &'a self,
    ) -> (RcRefCellInner<&'a mut u64>, RcRefCellInner<&'a mut [u8]>) {
        let lamports_inner = RcRefCellInner::new(self.unchecked_borrow_mut_lamports());
        let data_inner = RcRefCellInner::new(self.unchecked_borrow_mut_data());
        (lamports_inner, data_inner)
    }

    pub unsafe fn info_with<'a>(
        &'a self,
        lamports_data: &'a (RcRefCellInner<&'a mut u64>, RcRefCellInner<&'a mut [u8]>),
    ) -> AccountInfo<'a> {
        let (lamports, data) = lamports_data;
        AccountInfo {
            key: self.key(),
            lamports: unsafe { lamports.as_rcrc() },
            data: unsafe { data.as_rcrc() },
            owner: self.owner(),
            rent_epoch: u64::MAX,
            is_signer: self.is_signer(),
            is_writable: self.is_writable(),
            executable: self.executable(),
        }
    }

    #[inline(always)]
    pub fn key(&self) -> &Pubkey {
        unsafe { &(*self.inner).key }
    }
    #[inline(always)]
    pub fn owner(&self) -> &Pubkey {
        unsafe { &(*self.inner).owner }
    }
    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        unsafe { (*self.inner).is_signer != 0 }
    }
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        unsafe { (*self.inner).is_writable != 0 }
    }
    #[inline(always)]
    pub fn executable(&self) -> bool {
        unsafe { (*self.inner).executable != 0 }
    }
    #[inline(always)]
    pub fn data_len(&self) -> usize {
        unsafe { (*self.inner).data_len }
    }

    pub unsafe fn unchecked_borrow_lamports(&self) -> &u64 {
        &(*self.inner).lamports
    }
    pub unsafe fn unchecked_borrow_mut_lamports(&self) -> &mut u64 {
        &mut (*self.inner).lamports
    }
    pub unsafe fn unchecked_borrow_data(&self) -> &[u8] {
        core::slice::from_raw_parts(self.data_ptr(), (*self.inner).data_len)
    }
    pub unsafe fn unchecked_borrow_mut_data(&self) -> &mut [u8] {
        core::slice::from_raw_parts_mut(self.data_ptr(), (*self.inner).data_len)
    }

    pub fn try_borrow_lamports(&self) -> Option<Ref<u64>> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if mutable borrow is already taken
        if *borrow_state & 0b_1000_0000 != 0 {
            return None;
        }

        // Check if we have reached the max immutable borrow count
        if *borrow_state & 0b_0111_0000 == 0b_0111_0000 {
            return None;
        }

        // Increment the immutable borrow count
        *borrow_state += 1 << 4;

        // Return the reference to lamports
        Some(Ref {
            value: unsafe { &(*self.inner).lamports },
            state: unsafe { NonNull::new_unchecked(&mut (*self.inner).borrow_state) },
            is_lamport: true,
        })
    }

    pub fn try_borrow_mut_lamports(&self) -> Option<RefMut<u64>> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if any borrow (mutable or immutable) is already taken for lamports
        if *borrow_state & 0b_1111_0000 != 0 {
            return None;
        }

        // Set the mutable lamport borrow flag
        *borrow_state |= 0b_1000_0000;

        // Return the mutable reference to lamports
        Some(RefMut {
            value: unsafe { &mut (*self.inner).lamports },
            state: unsafe { NonNull::new_unchecked(&mut (*self.inner).borrow_state) },
            is_lamport: true,
        })
    }

    pub fn try_borrow_data(&self) -> Option<Ref<[u8]>> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if mutable data borrow is already taken (most significant bit of the data_borrow_state)
        if *borrow_state & 0b_0000_1000 != 0 {
            return None;
        }

        // Check if we have reached the max immutable data borrow count (7)
        if *borrow_state & 0b0111 == 0b0111 {
            return None;
        }

        // Increment the immutable data borrow count
        *borrow_state += 1;

        // Return the reference to data
        Some(Ref {
            value: unsafe { core::slice::from_raw_parts(self.data_ptr(), (*self.inner).data_len) },
            state: unsafe { NonNull::new_unchecked(&mut (*self.inner).borrow_state) },
            is_lamport: false,
        })
    }

    pub fn try_borrow_mut_data(&self) -> Option<RefMut<[u8]>> {
        let borrow_state = unsafe { &mut (*self.inner).borrow_state };

        // Check if any borrow (mutable or immutable) is already taken for data
        if *borrow_state & 0b_0000_1111 != 0 {
            return None;
        }

        // Set the mutable data borrow flag
        *borrow_state |= 0b0000_1000;

        assert_eq!(self.data_ptr() as usize % 8, 0); // TODO REMOVE

        // Return the mutable reference to data
        Some(RefMut {
            value: unsafe {
                core::slice::from_raw_parts_mut(self.data_ptr(), (*self.inner).data_len)
            },
            state: unsafe { NonNull::new_unchecked(&mut (*self.inner).borrow_state) },
            is_lamport: false,
        })
    }

    // private
    fn data_ptr(&self) -> *mut u8 {
        unsafe { (self.inner as *const _ as *mut u8).add(size_of::<NoStdAccountInfo4Inner>()) }
    }
}
