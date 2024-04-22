#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use pop_api::{
    primitives::{AccountId as AccountId32, AssetId},
    assets::fungibles::*,
};
pub enum FungiblesError {
    /// Not enough balance to fulfill a request is available.
    InsufficientBalance,
    /// Not enough allowance to fulfill a request is available.
    InsufficientAllowance,
    /// The asset status is not the expected status.
    IncorrectStatus,
    /// The asset ID is already taken.
    InUse,
    /// Minimum balance should be non-zero.
    MinBalanceZero,
    /// The signing account has no permission to do the operation.
    NoPermission,
    /// The given asset ID is unknown.
    Unknown,
    /// Recipient's address is zero.
    ZeroRecipientAddress,
    /// Sender's address is zero.
    ZeroSenderAddress,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for FungiblesError {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(
                    ::scale_info::Path::new_with_replace(
                        "FungiblesError",
                        "fungibles",
                        &[],
                    ),
                )
                .type_params(::alloc::vec::Vec::new())
                .variant(
                    ::scale_info::build::Variants::new()
                        .variant(
                            "InsufficientBalance",
                            |v| {
                                v
                                    .index(0usize as ::core::primitive::u8)
                                    .docs(
                                        &["Not enough balance to fulfill a request is available."],
                                    )
                            },
                        )
                        .variant(
                            "InsufficientAllowance",
                            |v| {
                                v
                                    .index(1usize as ::core::primitive::u8)
                                    .docs(
                                        &["Not enough allowance to fulfill a request is available."],
                                    )
                            },
                        )
                        .variant(
                            "IncorrectStatus",
                            |v| {
                                v
                                    .index(2usize as ::core::primitive::u8)
                                    .docs(&["The asset status is not the expected status."])
                            },
                        )
                        .variant(
                            "InUse",
                            |v| {
                                v
                                    .index(3usize as ::core::primitive::u8)
                                    .docs(&["The asset ID is already taken."])
                            },
                        )
                        .variant(
                            "MinBalanceZero",
                            |v| {
                                v
                                    .index(4usize as ::core::primitive::u8)
                                    .docs(&["Minimum balance should be non-zero."])
                            },
                        )
                        .variant(
                            "NoPermission",
                            |v| {
                                v
                                    .index(5usize as ::core::primitive::u8)
                                    .docs(
                                        &[
                                            "The signing account has no permission to do the operation.",
                                        ],
                                    )
                            },
                        )
                        .variant(
                            "Unknown",
                            |v| {
                                v
                                    .index(6usize as ::core::primitive::u8)
                                    .docs(&["The given asset ID is unknown."])
                            },
                        )
                        .variant(
                            "ZeroRecipientAddress",
                            |v| {
                                v
                                    .index(7usize as ::core::primitive::u8)
                                    .docs(&["Recipient's address is zero."])
                            },
                        )
                        .variant(
                            "ZeroSenderAddress",
                            |v| {
                                v
                                    .index(8usize as ::core::primitive::u8)
                                    .docs(&["Sender's address is zero."])
                            },
                        ),
                )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for FungiblesError {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                FungiblesError::InsufficientBalance => "InsufficientBalance",
                FungiblesError::InsufficientAllowance => "InsufficientAllowance",
                FungiblesError::IncorrectStatus => "IncorrectStatus",
                FungiblesError::InUse => "InUse",
                FungiblesError::MinBalanceZero => "MinBalanceZero",
                FungiblesError::NoPermission => "NoPermission",
                FungiblesError::Unknown => "Unknown",
                FungiblesError::ZeroRecipientAddress => "ZeroRecipientAddress",
                FungiblesError::ZeroSenderAddress => "ZeroSenderAddress",
            },
        )
    }
}
#[automatically_derived]
impl ::core::marker::Copy for FungiblesError {}
#[automatically_derived]
impl ::core::clone::Clone for FungiblesError {
    #[inline]
    fn clone(&self) -> FungiblesError {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for FungiblesError {}
#[automatically_derived]
impl ::core::cmp::PartialEq for FungiblesError {
    #[inline]
    fn eq(&self, other: &FungiblesError) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for FungiblesError {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::scale::Encode for FungiblesError {
        fn size_hint(&self) -> usize {
            1_usize
                + match *self {
                    FungiblesError::InsufficientBalance => 0_usize,
                    FungiblesError::InsufficientAllowance => 0_usize,
                    FungiblesError::IncorrectStatus => 0_usize,
                    FungiblesError::InUse => 0_usize,
                    FungiblesError::MinBalanceZero => 0_usize,
                    FungiblesError::NoPermission => 0_usize,
                    FungiblesError::Unknown => 0_usize,
                    FungiblesError::ZeroRecipientAddress => 0_usize,
                    FungiblesError::ZeroSenderAddress => 0_usize,
                    _ => 0_usize,
                }
        }
        fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            match *self {
                FungiblesError::InsufficientBalance => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                }
                FungiblesError::InsufficientAllowance => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                }
                FungiblesError::IncorrectStatus => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                }
                FungiblesError::InUse => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                }
                FungiblesError::MinBalanceZero => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                }
                FungiblesError::NoPermission => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                }
                FungiblesError::Unknown => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                }
                FungiblesError::ZeroRecipientAddress => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                }
                FungiblesError::ZeroSenderAddress => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                }
                _ => {}
            }
        }
    }
    #[automatically_derived]
    impl ::scale::EncodeLike for FungiblesError {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::scale::Decode for FungiblesError {
        fn decode<__CodecInputEdqy: ::scale::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::scale::Error> {
            match __codec_input_edqy
                .read_byte()
                .map_err(|e| {
                    e
                        .chain(
                            "Could not decode `FungiblesError`, failed to read variant byte",
                        )
                })?
            {
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::InsufficientBalance)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::InsufficientAllowance)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 2usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::IncorrectStatus)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 3usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::InUse)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 4usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::MinBalanceZero)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 5usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::NoPermission)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 6usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::Unknown)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 7usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::ZeroRecipientAddress)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 8usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(FungiblesError::ZeroSenderAddress)
                    })();
                }
                _ => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Err(
                            <_ as ::core::convert::Into<
                                _,
                            >>::into(
                                "Could not decode `FungiblesError`, variant doesn't exist",
                            ),
                        )
                    })();
                }
            }
        }
    }
};
impl From<Error> for FungiblesError {
    fn from(error: Error) -> Self {
        match error {
            Error::InUse => FungiblesError::InUse,
            Error::MinBalanceZero => FungiblesError::MinBalanceZero,
            Error::Unknown => FungiblesError::Unknown,
            _ => ::core::panicking::panic("not yet implemented"),
        }
    }
}
/// The fungibles result type.
pub type Result<T> = core::result::Result<T, FungiblesError>;
mod fungibles {
    impl ::ink::env::ContractEnv for Fungibles {
        type Env = pop_api::Environment;
    }
    type Environment = <Fungibles as ::ink::env::ContractEnv>::Env;
    type AccountId = <<Fungibles as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::AccountId;
    type Balance = <<Fungibles as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Balance;
    type Hash = <<Fungibles as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Hash;
    type Timestamp = <<Fungibles as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::Timestamp;
    type BlockNumber = <<Fungibles as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::BlockNumber;
    type ChainExtension = <<Fungibles as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::ChainExtension;
    const MAX_EVENT_TOPICS: usize = <<Fungibles as ::ink::env::ContractEnv>::Env as ::ink::env::Environment>::MAX_EVENT_TOPICS;
    const _: () = {
        struct Check {
            salt: (),
        }
    };
    #[scale_info(crate = ::ink::scale_info)]
    #[cfg(not(feature = "__ink_dylint_Storage"))]
    pub struct Fungibles {}
    const _: () = {
        impl<
            __ink_generic_salt: ::ink::storage::traits::StorageKey,
        > ::ink::storage::traits::StorableHint<__ink_generic_salt> for Fungibles {
            type Type = Fungibles;
            type PreferredKey = ::ink::storage::traits::AutoKey;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageKey for Fungibles {
            const KEY: ::ink::primitives::Key = <() as ::ink::storage::traits::StorageKey>::KEY;
        }
    };
    const _: () = {
        impl ::ink::storage::traits::Storable for Fungibles {
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn decode<__ink_I: ::ink::scale::Input>(
                __input: &mut __ink_I,
            ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                ::core::result::Result::Ok(Fungibles {})
            }
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn encode<__ink_O: ::ink::scale::Output + ?::core::marker::Sized>(
                &self,
                __dest: &mut __ink_O,
            ) {
                match self {
                    Fungibles {} => {}
                }
            }
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn encoded_size(&self) -> ::core::primitive::usize {
                match self {
                    Fungibles {} => ::core::primitive::usize::MIN,
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::ink::scale_info::TypeInfo for Fungibles {
            type Identity = Self;
            fn type_info() -> ::ink::scale_info::Type {
                ::ink::scale_info::Type::builder()
                    .path(
                        ::ink::scale_info::Path::new_with_replace(
                            "Fungibles",
                            "fungibles::fungibles",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(::ink::scale_info::build::Fields::named())
            }
        }
    };
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for Fungibles {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new("Fungibles", []),
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::default::Default for Fungibles {
        #[inline]
        fn default() -> Fungibles {
            Fungibles {}
        }
    }
    const _: () = {
        impl ::ink::reflect::ContractName for Fungibles {
            const NAME: &'static str = "Fungibles";
        }
    };
    const _: () = {
        impl<'a> ::ink::codegen::Env for &'a Fungibles {
            type EnvAccess = ::ink::EnvAccess<
                'a,
                <Fungibles as ::ink::env::ContractEnv>::Env,
            >;
            fn env(self) -> Self::EnvAccess {
                <<Self as ::ink::codegen::Env>::EnvAccess as ::core::default::Default>::default()
            }
        }
        impl<'a> ::ink::codegen::StaticEnv for Fungibles {
            type EnvAccess = ::ink::EnvAccess<
                'static,
                <Fungibles as ::ink::env::ContractEnv>::Env,
            >;
            fn env() -> Self::EnvAccess {
                <<Self as ::ink::codegen::StaticEnv>::EnvAccess as ::core::default::Default>::default()
            }
        }
    };
    const _: () = {
        #[allow(unused_imports)]
        use ::ink::codegen::{Env as _, StaticEnv as _};
    };
    impl ::ink::reflect::DispatchableConstructorInfo<0x9BAE9D5E_u32> for Fungibles {
        type Input = ();
        type Output = Self;
        type Storage = Fungibles;
        type Error = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Fungibles>>::Error;
        const IS_RESULT: ::core::primitive::bool = <::ink::reflect::ConstructorOutputValue<
            Self,
        > as ::ink::reflect::ConstructorOutput<Fungibles>>::IS_RESULT;
        const CALLABLE: fn(Self::Input) -> Self::Output = |_| { Fungibles::new() };
        const PAYABLE: ::core::primitive::bool = true;
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x9B_u8,
            0xAE_u8,
            0x9D_u8,
            0x5E_u8,
        ];
        const LABEL: &'static ::core::primitive::str = "new";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0xDB6375A8_u32> for Fungibles {
        type Input = AssetId;
        type Output = Result<Balance>;
        type Storage = Fungibles;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { Fungibles::total_supply(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xDB_u8,
            0x63_u8,
            0x75_u8,
            0xA8_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "total_supply";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x0F755A56_u32> for Fungibles {
        type Input = (AssetId, AccountId32);
        type Output = Result<Balance>;
        type Storage = Fungibles;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            (__ink_binding_0, __ink_binding_1)|
        { Fungibles::balance_of(storage, __ink_binding_0, __ink_binding_1) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x0F_u8,
            0x75_u8,
            0x5A_u8,
            0x56_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "balance_of";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x6A00165E_u32> for Fungibles {
        type Input = (AssetId, AccountId32, AccountId32);
        type Output = Result<Balance>;
        type Storage = Fungibles;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            (__ink_binding_0, __ink_binding_1, __ink_binding_2)|
        {
            Fungibles::allowance(
                storage,
                __ink_binding_0,
                __ink_binding_1,
                __ink_binding_2,
            )
        };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x6A_u8,
            0x00_u8,
            0x16_u8,
            0x5E_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "allowance";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0xAA6B65DB_u32> for Fungibles {
        type Input = AssetId;
        type Output = Result<bool>;
        type Storage = Fungibles;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            __ink_binding_0|
        { Fungibles::asset_exists(storage, __ink_binding_0) };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0xAA_u8,
            0x6B_u8,
            0x65_u8,
            0xDB_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "asset_exists";
    }
    impl ::ink::reflect::DispatchableMessageInfo<0x1F8E8E22_u32> for Fungibles {
        type Input = (u32, AccountId32, Balance);
        type Output = Result<()>;
        type Storage = Fungibles;
        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output = |
            storage,
            (__ink_binding_0, __ink_binding_1, __ink_binding_2)|
        {
            Fungibles::mint_asset(
                storage,
                __ink_binding_0,
                __ink_binding_1,
                __ink_binding_2,
            )
        };
        const SELECTOR: [::core::primitive::u8; 4usize] = [
            0x1F_u8,
            0x8E_u8,
            0x8E_u8,
            0x22_u8,
        ];
        const PAYABLE: ::core::primitive::bool = false;
        const MUTATES: ::core::primitive::bool = false;
        const LABEL: &'static ::core::primitive::str = "mint_asset";
    }
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_ConstructorDecoder {
            Constructor0(
                <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                    0x9BAE9D5E_u32,
                >>::Input,
            ),
        }
        impl ::ink::reflect::DecodeDispatch for __ink_ConstructorDecoder {
            fn decode_dispatch<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::reflect::DispatchError>
            where
                I: ::ink::scale::Input,
            {
                const CONSTRUCTOR_0: [::core::primitive::u8; 4usize] = <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                    0x9BAE9D5E_u32,
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::ink::scale::Decode>::decode(
                        input,
                    )
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    CONSTRUCTOR_0 => {
                        ::core::result::Result::Ok(
                            Self::Constructor0(
                                <<Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                                    0x9BAE9D5E_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    _invalid => {
                        ::core::result::Result::Err(
                            ::ink::reflect::DispatchError::UnknownSelector,
                        )
                    }
                }
            }
        }
        impl ::ink::scale::Decode for __ink_ConstructorDecoder {
            fn decode<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::scale::Error>
            where
                I: ::ink::scale::Input,
            {
                <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                    .map_err(::core::convert::Into::into)
            }
        }
        impl ::ink::reflect::ExecuteDispatchable for __ink_ConstructorDecoder {
            #[allow(clippy::nonminimal_bool)]
            fn execute_dispatchable(
                self,
            ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                match self {
                    Self::Constructor0(input) => {
                        if {
                            false
                                || {
                                    let constructor_0 = false;
                                    let constructor_0 = <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                                        0x9BAE9D5E_u32,
                                    >>::PAYABLE;
                                    constructor_0
                                }
                        }
                            && !<Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                                0x9BAE9D5E_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Fungibles as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::Output = <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                            0x9BAE9D5E_u32,
                        >>::CALLABLE(input);
                        let output_value = ::ink::reflect::ConstructorOutputValue::new(
                            result,
                        );
                        let output_result = <::ink::reflect::ConstructorOutputValue<
                            <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                                0x9BAE9D5E_u32,
                            >>::Output,
                        > as ::ink::reflect::ConstructorOutput<
                            Fungibles,
                        >>::as_result(&output_value);
                        if let ::core::result::Result::Ok(contract) = output_result
                            .as_ref()
                        {
                            ::ink::env::set_contract_storage::<
                                ::ink::primitives::Key,
                                Fungibles,
                            >(
                                &<Fungibles as ::ink::storage::traits::StorageKey>::KEY,
                                contract,
                            );
                        }
                        let mut flag = ::ink::env::ReturnFlags::empty();
                        if output_result.is_err() {
                            flag = ::ink::env::ReturnFlags::REVERT;
                        }
                        ::ink::env::return_value::<
                            ::ink::ConstructorResult<
                                ::core::result::Result<
                                    (),
                                    &<::ink::reflect::ConstructorOutputValue<
                                        <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                                            0x9BAE9D5E_u32,
                                        >>::Output,
                                    > as ::ink::reflect::ConstructorOutput<Fungibles>>::Error,
                                >,
                            >,
                        >(
                            flag,
                            &::ink::ConstructorResult::Ok(output_result.map(|_| ())),
                        );
                    }
                }
            }
        }
        impl ::ink::reflect::ContractConstructorDecoder for Fungibles {
            type Type = __ink_ConstructorDecoder;
        }
    };
    const _: () = {
        #[allow(non_camel_case_types)]
        pub enum __ink_MessageDecoder {
            Message0(
                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0xDB6375A8_u32,
                >>::Input,
            ),
            Message1(
                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0x0F755A56_u32,
                >>::Input,
            ),
            Message2(
                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0x6A00165E_u32,
                >>::Input,
            ),
            Message3(
                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0xAA6B65DB_u32,
                >>::Input,
            ),
            Message4(
                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0x1F8E8E22_u32,
                >>::Input,
            ),
        }
        impl ::ink::reflect::DecodeDispatch for __ink_MessageDecoder {
            fn decode_dispatch<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::reflect::DispatchError>
            where
                I: ::ink::scale::Input,
            {
                const MESSAGE_0: [::core::primitive::u8; 4usize] = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0xDB6375A8_u32,
                >>::SELECTOR;
                const MESSAGE_1: [::core::primitive::u8; 4usize] = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0x0F755A56_u32,
                >>::SELECTOR;
                const MESSAGE_2: [::core::primitive::u8; 4usize] = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0x6A00165E_u32,
                >>::SELECTOR;
                const MESSAGE_3: [::core::primitive::u8; 4usize] = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0xAA6B65DB_u32,
                >>::SELECTOR;
                const MESSAGE_4: [::core::primitive::u8; 4usize] = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                    0x1F8E8E22_u32,
                >>::SELECTOR;
                match <[::core::primitive::u8; 4usize] as ::ink::scale::Decode>::decode(
                        input,
                    )
                    .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                {
                    MESSAGE_0 => {
                        ::core::result::Result::Ok(
                            Self::Message0(
                                <<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xDB6375A8_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_1 => {
                        ::core::result::Result::Ok(
                            Self::Message1(
                                <<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x0F755A56_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_2 => {
                        ::core::result::Result::Ok(
                            Self::Message2(
                                <<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x6A00165E_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_3 => {
                        ::core::result::Result::Ok(
                            Self::Message3(
                                <<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xAA6B65DB_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    MESSAGE_4 => {
                        ::core::result::Result::Ok(
                            Self::Message4(
                                <<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x1F8E8E22_u32,
                                >>::Input as ::ink::scale::Decode>::decode(input)
                                    .map_err(|_| {
                                        ::ink::reflect::DispatchError::InvalidParameters
                                    })?,
                            ),
                        )
                    }
                    _invalid => {
                        ::core::result::Result::Err(
                            ::ink::reflect::DispatchError::UnknownSelector,
                        )
                    }
                }
            }
        }
        impl ::ink::scale::Decode for __ink_MessageDecoder {
            fn decode<I>(
                input: &mut I,
            ) -> ::core::result::Result<Self, ::ink::scale::Error>
            where
                I: ::ink::scale::Input,
            {
                <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                    .map_err(::core::convert::Into::into)
            }
        }
        fn push_contract(contract: ::core::mem::ManuallyDrop<Fungibles>, mutates: bool) {
            if mutates {
                ::ink::env::set_contract_storage::<
                    ::ink::primitives::Key,
                    Fungibles,
                >(&<Fungibles as ::ink::storage::traits::StorageKey>::KEY, &contract);
            }
        }
        impl ::ink::reflect::ExecuteDispatchable for __ink_MessageDecoder {
            #[allow(clippy::nonminimal_bool, clippy::let_unit_value)]
            fn execute_dispatchable(
                self,
            ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                let key = <Fungibles as ::ink::storage::traits::StorageKey>::KEY;
                let mut contract: ::core::mem::ManuallyDrop<Fungibles> = ::core::mem::ManuallyDrop::new(
                    match ::ink::env::get_contract_storage(&key) {
                        ::core::result::Result::Ok(
                            ::core::option::Option::Some(value),
                        ) => value,
                        ::core::result::Result::Ok(::core::option::Option::None) => {
                            ::core::panicking::panic_fmt(
                                format_args!("storage entry was empty"),
                            );
                        }
                        ::core::result::Result::Err(_) => {
                            ::core::panicking::panic_fmt(
                                format_args!("could not properly decode storage entry"),
                            );
                        }
                    },
                );
                match self {
                    Self::Message0(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xDB6375A8_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x0F755A56_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x6A00165E_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xAA6B65DB_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x1F8E8E22_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                0xDB6375A8_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Fungibles as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0xDB6375A8_u32,
                        >>::Output = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0xDB6375A8_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xDB6375A8_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xDB6375A8_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xDB6375A8_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message1(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xDB6375A8_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x0F755A56_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x6A00165E_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xAA6B65DB_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x1F8E8E22_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                0x0F755A56_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Fungibles as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0x0F755A56_u32,
                        >>::Output = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0x0F755A56_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x0F755A56_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x0F755A56_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x0F755A56_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message2(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xDB6375A8_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x0F755A56_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x6A00165E_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xAA6B65DB_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x1F8E8E22_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                0x6A00165E_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Fungibles as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0x6A00165E_u32,
                        >>::Output = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0x6A00165E_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x6A00165E_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x6A00165E_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x6A00165E_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message3(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xDB6375A8_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x0F755A56_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x6A00165E_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xAA6B65DB_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x1F8E8E22_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                0xAA6B65DB_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Fungibles as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0xAA6B65DB_u32,
                        >>::Output = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0xAA6B65DB_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xAA6B65DB_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xAA6B65DB_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0xAA6B65DB_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                    Self::Message4(input) => {
                        if {
                            false
                                || {
                                    let message_0 = false;
                                    let message_0 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xDB6375A8_u32,
                                    >>::PAYABLE;
                                    message_0
                                }
                                || {
                                    let message_1 = false;
                                    let message_1 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x0F755A56_u32,
                                    >>::PAYABLE;
                                    message_1
                                }
                                || {
                                    let message_2 = false;
                                    let message_2 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x6A00165E_u32,
                                    >>::PAYABLE;
                                    message_2
                                }
                                || {
                                    let message_3 = false;
                                    let message_3 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0xAA6B65DB_u32,
                                    >>::PAYABLE;
                                    message_3
                                }
                                || {
                                    let message_4 = false;
                                    let message_4 = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                        0x1F8E8E22_u32,
                                    >>::PAYABLE;
                                    message_4
                                }
                        }
                            && !<Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                0x1F8E8E22_u32,
                            >>::PAYABLE
                        {
                            ::ink::codegen::deny_payment::<
                                <Fungibles as ::ink::env::ContractEnv>::Env,
                            >()?;
                        }
                        let result: <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0x1F8E8E22_u32,
                        >>::Output = <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                            0x1F8E8E22_u32,
                        >>::CALLABLE(&mut contract, input);
                        let is_reverted = {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x1F8E8E22_u32,
                                >>::Output,
                            >::VALUE
                        }
                            && {
                                #[allow(unused_imports)]
                                use ::ink::result_info::IsResultErrFallback as _;
                                ::ink::result_info::IsResultErr(&result).value()
                            };
                        let mut flag = ::ink::env::ReturnFlags::REVERT;
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(
                                contract,
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x1F8E8E22_u32,
                                >>::MUTATES,
                            );
                        }
                        ::ink::env::return_value::<
                            ::ink::MessageResult<
                                <Fungibles as ::ink::reflect::DispatchableMessageInfo<
                                    0x1F8E8E22_u32,
                                >>::Output,
                            >,
                        >(flag, &::ink::MessageResult::Ok(result))
                    }
                };
            }
        }
        impl ::ink::reflect::ContractMessageDecoder for Fungibles {
            type Type = __ink_MessageDecoder;
        }
    };
    const _: () = {
        use ::ink::codegen::{Env as _, StaticEnv as _};
        const _: ::ink::codegen::utils::IsSameType<Fungibles> = ::ink::codegen::utils::IsSameType::<
            Fungibles,
        >::new();
        impl Fungibles {
            #[cfg(not(feature = "__ink_dylint_Constructor"))]
            pub fn new() -> Self {
                ::ink_env::debug_message(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "{0}\n",
                                {
                                    let res = ::alloc::fmt::format(
                                        format_args!("PopApiAssetsExample::new"),
                                    );
                                    res
                                },
                            ),
                        );
                        res
                    },
                );
                Default::default()
            }
            pub fn total_supply(&self, id: AssetId) -> Result<Balance> {
                total_supply(id).map_err(From::from)
            }
            pub fn balance_of(
                &self,
                id: AssetId,
                owner: AccountId32,
            ) -> Result<Balance> {
                balance_of(id, owner).map_err(From::from)
            }
            pub fn allowance(
                &self,
                id: AssetId,
                owner: AccountId32,
                spender: AccountId32,
            ) -> Result<Balance> {
                allowance(id, owner, spender).map_err(From::from)
            }
            pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
                asset_exists(id).map_err(From::from)
            }
            pub fn mint_asset(
                &self,
                id: u32,
                beneficiary: AccountId32,
                amount: Balance,
            ) -> Result<()> {
                ::ink_env::debug_message(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "{0}\n",
                                {
                                    let res = ::alloc::fmt::format(
                                        format_args!(
                                            "PopApiAssetsExample::mint_asset_through_runtime: id: {0:?} beneficiary: {1:?} amount: {2:?}",
                                            id,
                                            beneficiary,
                                            amount,
                                        ),
                                    );
                                    res
                                },
                            ),
                        );
                        res
                    },
                );
                let result = mint(id, beneficiary, amount)?;
                ::ink_env::debug_message(
                    &{
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "{0}\n",
                                {
                                    let res = ::alloc::fmt::format(
                                        format_args!("Result: {0:?}", result),
                                    );
                                    res
                                },
                            ),
                        );
                        res
                    },
                );
                Ok(())
            }
        }
        const _: () = {
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AssetId>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<Result<Balance>>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AssetId>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AccountId32>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<Result<Balance>>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AssetId>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AccountId32>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AccountId32>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<Result<Balance>>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AssetId>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<Result<bool>>,
            >();
            ::ink::codegen::utils::consume_type::<::ink::codegen::DispatchInput<u32>>();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<AccountId32>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchInput<Balance>,
            >();
            ::ink::codegen::utils::consume_type::<
                ::ink::codegen::DispatchOutput<Result<()>>,
            >();
        };
    };
    const _: () = {
        #[codec(crate = ::ink::scale)]
        #[scale_info(crate = ::ink::scale_info)]
        /// The ink! smart contract's call builder.
        ///
        /// Implements the underlying on-chain calling of the ink! smart contract
        /// messages and trait implementations in a type safe way.
        #[repr(transparent)]
        pub struct CallBuilder {
            account_id: AccountId,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::ink::scale_info::TypeInfo for CallBuilder {
                type Identity = Self;
                fn type_info() -> ::ink::scale_info::Type {
                    ::ink::scale_info::Type::builder()
                        .path(
                            ::ink::scale_info::Path::new_with_replace(
                                "CallBuilder",
                                "fungibles::fungibles",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .docs(
                            &[
                                "The ink! smart contract's call builder.",
                                "",
                                "Implements the underlying on-chain calling of the ink! smart contract",
                                "messages and trait implementations in a type safe way.",
                            ],
                        )
                        .composite(
                            ::ink::scale_info::build::Fields::named()
                                .field(|f| {
                                    f
                                        .ty::<AccountId>()
                                        .name("account_id")
                                        .type_name("AccountId")
                                }),
                        )
                }
            }
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::ink::scale::Decode for CallBuilder {
                fn decode<__CodecInputEdqy: ::ink::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                    ::core::result::Result::Ok(CallBuilder {
                        account_id: {
                            let __codec_res_edqy = <AccountId as ::ink::scale::Decode>::decode(
                                __codec_input_edqy,
                            );
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `CallBuilder::account_id`"),
                                    );
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                    __codec_res_edqy
                                }
                            }
                        },
                    })
                }
                fn decode_into<__CodecInputEdqy: ::ink::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                    dst_: &mut ::core::mem::MaybeUninit<Self>,
                ) -> ::core::result::Result<
                    ::ink::scale::DecodeFinished,
                    ::ink::scale::Error,
                > {
                    match (
                        &::core::mem::size_of::<AccountId>(),
                        &::core::mem::size_of::<Self>(),
                    ) {
                        (left_val, right_val) => {
                            if !(*left_val == *right_val) {
                                let kind = ::core::panicking::AssertKind::Eq;
                                ::core::panicking::assert_failed(
                                    kind,
                                    &*left_val,
                                    &*right_val,
                                    ::core::option::Option::None,
                                );
                            }
                        }
                    };
                    if !(if ::core::mem::size_of::<AccountId>() > 0 { 1 } else { 0 }
                        <= 1)
                    {
                        ::core::panicking::panic(
                            "assertion failed: if ::core::mem::size_of::<AccountId>() > 0 { 1 } else { 0 } <= 1",
                        )
                    }
                    {
                        let dst_: &mut ::core::mem::MaybeUninit<Self> = dst_;
                        let dst_: &mut ::core::mem::MaybeUninit<AccountId> = unsafe {
                            &mut *dst_
                                .as_mut_ptr()
                                .cast::<::core::mem::MaybeUninit<AccountId>>()
                        };
                        <AccountId as ::ink::scale::Decode>::decode_into(
                            __codec_input_edqy,
                            dst_,
                        )?;
                    }
                    unsafe {
                        ::core::result::Result::Ok(
                            ::ink::scale::DecodeFinished::assert_decoding_finished(),
                        )
                    }
                }
            }
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::ink::scale::Encode for CallBuilder {
                fn size_hint(&self) -> usize {
                    ::ink::scale::Encode::size_hint(&&self.account_id)
                }
                fn encode_to<
                    __CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    ::ink::scale::Encode::encode_to(&&self.account_id, __codec_dest_edqy)
                }
                fn encode(
                    &self,
                ) -> ::ink::scale::alloc::vec::Vec<::core::primitive::u8> {
                    ::ink::scale::Encode::encode(&&self.account_id)
                }
                fn using_encoded<
                    __CodecOutputReturn,
                    __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                            &[::core::primitive::u8],
                        ) -> __CodecOutputReturn,
                >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
                    ::ink::scale::Encode::using_encoded(&&self.account_id, f)
                }
            }
            #[automatically_derived]
            impl ::ink::scale::EncodeLike for CallBuilder {}
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for CallBuilder {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "CallBuilder",
                    "account_id",
                    &&self.account_id,
                )
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for CallBuilder {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.account_id, state)
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for CallBuilder {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for CallBuilder {
            #[inline]
            fn eq(&self, other: &CallBuilder) -> bool {
                self.account_id == other.account_id
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for CallBuilder {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<AccountId>;
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CallBuilder {
            #[inline]
            fn clone(&self) -> CallBuilder {
                CallBuilder {
                    account_id: ::core::clone::Clone::clone(&self.account_id),
                }
            }
        }
        const _: () = {
            impl ::ink::storage::traits::StorageLayout for CallBuilder {
                fn layout(
                    __key: &::ink::primitives::Key,
                ) -> ::ink::metadata::layout::Layout {
                    ::ink::metadata::layout::Layout::Struct(
                        ::ink::metadata::layout::StructLayout::new(
                            "CallBuilder",
                            [
                                ::ink::metadata::layout::FieldLayout::new(
                                    "account_id",
                                    <AccountId as ::ink::storage::traits::StorageLayout>::layout(
                                        __key,
                                    ),
                                ),
                            ],
                        ),
                    )
                }
            }
        };
        const _: () = {
            impl ::ink::codegen::ContractCallBuilder for Fungibles {
                type Type = CallBuilder;
            }
            impl ::ink::env::ContractEnv for CallBuilder {
                type Env = <Fungibles as ::ink::env::ContractEnv>::Env;
            }
        };
        impl ::ink::env::call::FromAccountId<Environment> for CallBuilder {
            #[inline]
            fn from_account_id(account_id: AccountId) -> Self {
                Self { account_id }
            }
        }
        impl ::ink::ToAccountId<Environment> for CallBuilder {
            #[inline]
            fn to_account_id(&self) -> AccountId {
                <AccountId as ::core::clone::Clone>::clone(&self.account_id)
            }
        }
        impl ::core::convert::AsRef<AccountId> for CallBuilder {
            fn as_ref(&self) -> &AccountId {
                &self.account_id
            }
        }
        impl ::core::convert::AsMut<AccountId> for CallBuilder {
            fn as_mut(&mut self) -> &mut AccountId {
                &mut self.account_id
            }
        }
        impl CallBuilder {
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn total_supply(
                &self,
                __ink_binding_0: AssetId,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<AssetId>,
                            ::ink::env::call::utils::EmptyArgumentList,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::utils::ReturnType<Result<Balance>>,
                >,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0xDB_u8,
                                    0x63_u8,
                                    0x75_u8,
                                    0xA8_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0),
                    )
                    .returns::<Result<Balance>>()
            }
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn balance_of(
                &self,
                __ink_binding_0: AssetId,
                __ink_binding_1: AccountId32,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<AccountId32>,
                            ::ink::env::call::utils::ArgumentList<
                                ::ink::env::call::utils::Argument<AssetId>,
                                ::ink::env::call::utils::EmptyArgumentList,
                            >,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::utils::ReturnType<Result<Balance>>,
                >,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0x0F_u8,
                                    0x75_u8,
                                    0x5A_u8,
                                    0x56_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0)
                            .push_arg(__ink_binding_1),
                    )
                    .returns::<Result<Balance>>()
            }
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn allowance(
                &self,
                __ink_binding_0: AssetId,
                __ink_binding_1: AccountId32,
                __ink_binding_2: AccountId32,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<AccountId32>,
                            ::ink::env::call::utils::ArgumentList<
                                ::ink::env::call::utils::Argument<AccountId32>,
                                ::ink::env::call::utils::ArgumentList<
                                    ::ink::env::call::utils::Argument<AssetId>,
                                    ::ink::env::call::utils::EmptyArgumentList,
                                >,
                            >,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::utils::ReturnType<Result<Balance>>,
                >,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0x6A_u8,
                                    0x00_u8,
                                    0x16_u8,
                                    0x5E_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0)
                            .push_arg(__ink_binding_1)
                            .push_arg(__ink_binding_2),
                    )
                    .returns::<Result<Balance>>()
            }
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn asset_exists(
                &self,
                __ink_binding_0: AssetId,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<AssetId>,
                            ::ink::env::call::utils::EmptyArgumentList,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::utils::ReturnType<Result<bool>>,
                >,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0xAA_u8,
                                    0x6B_u8,
                                    0x65_u8,
                                    0xDB_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0),
                    )
                    .returns::<Result<bool>>()
            }
            #[allow(clippy::type_complexity)]
            #[inline]
            pub fn mint_asset(
                &self,
                __ink_binding_0: u32,
                __ink_binding_1: AccountId32,
                __ink_binding_2: Balance,
            ) -> ::ink::env::call::CallBuilder<
                Environment,
                ::ink::env::call::utils::Set<::ink::env::call::Call<Environment>>,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::ExecutionInput<
                        ::ink::env::call::utils::ArgumentList<
                            ::ink::env::call::utils::Argument<Balance>,
                            ::ink::env::call::utils::ArgumentList<
                                ::ink::env::call::utils::Argument<AccountId32>,
                                ::ink::env::call::utils::ArgumentList<
                                    ::ink::env::call::utils::Argument<u32>,
                                    ::ink::env::call::utils::EmptyArgumentList,
                                >,
                            >,
                        >,
                    >,
                >,
                ::ink::env::call::utils::Set<
                    ::ink::env::call::utils::ReturnType<Result<()>>,
                >,
            > {
                ::ink::env::call::build_call::<Environment>()
                    .call(::ink::ToAccountId::to_account_id(self))
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                                ::ink::env::call::Selector::new([
                                    0x1F_u8,
                                    0x8E_u8,
                                    0x8E_u8,
                                    0x22_u8,
                                ]),
                            )
                            .push_arg(__ink_binding_0)
                            .push_arg(__ink_binding_1)
                            .push_arg(__ink_binding_2),
                    )
                    .returns::<Result<()>>()
            }
        }
    };
    #[codec(crate = ::ink::scale)]
    #[scale_info(crate = ::ink::scale_info)]
    pub struct FungiblesRef {
        inner: <Fungibles as ::ink::codegen::ContractCallBuilder>::Type,
    }
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl ::ink::scale_info::TypeInfo for FungiblesRef {
            type Identity = Self;
            fn type_info() -> ::ink::scale_info::Type {
                ::ink::scale_info::Type::builder()
                    .path(
                        ::ink::scale_info::Path::new_with_replace(
                            "FungiblesRef",
                            "fungibles::fungibles",
                            &[],
                        ),
                    )
                    .type_params(::alloc::vec::Vec::new())
                    .composite(
                        ::ink::scale_info::build::Fields::named()
                            .field(|f| {
                                f
                                    .ty::<
                                        <Fungibles as ::ink::codegen::ContractCallBuilder>::Type,
                                    >()
                                    .name("inner")
                                    .type_name(
                                        "<Fungibles as::ink::codegen::ContractCallBuilder>::Type",
                                    )
                            }),
                    )
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Decode for FungiblesRef {
            fn decode<__CodecInputEdqy: ::ink::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::ink::scale::Error> {
                ::core::result::Result::Ok(FungiblesRef {
                    inner: {
                        let __codec_res_edqy = <<Fungibles as ::ink::codegen::ContractCallBuilder>::Type as ::ink::scale::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            ::core::result::Result::Err(e) => {
                                return ::core::result::Result::Err(
                                    e.chain("Could not decode `FungiblesRef::inner`"),
                                );
                            }
                            ::core::result::Result::Ok(__codec_res_edqy) => {
                                __codec_res_edqy
                            }
                        }
                    },
                })
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::ink::scale::Encode for FungiblesRef {
            fn size_hint(&self) -> usize {
                ::ink::scale::Encode::size_hint(&&self.inner)
            }
            fn encode_to<
                __CodecOutputEdqy: ::ink::scale::Output + ?::core::marker::Sized,
            >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                ::ink::scale::Encode::encode_to(&&self.inner, __codec_dest_edqy)
            }
            fn encode(&self) -> ::ink::scale::alloc::vec::Vec<::core::primitive::u8> {
                ::ink::scale::Encode::encode(&&self.inner)
            }
            fn using_encoded<
                __CodecOutputReturn,
                __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                        &[::core::primitive::u8],
                    ) -> __CodecOutputReturn,
            >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
                ::ink::scale::Encode::using_encoded(&&self.inner, f)
            }
        }
        #[automatically_derived]
        impl ::ink::scale::EncodeLike for FungiblesRef {}
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for FungiblesRef {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "FungiblesRef",
                "inner",
                &&self.inner,
            )
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for FungiblesRef {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.inner, state)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for FungiblesRef {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for FungiblesRef {
        #[inline]
        fn eq(&self, other: &FungiblesRef) -> bool {
            self.inner == other.inner
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for FungiblesRef {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<
                <Fungibles as ::ink::codegen::ContractCallBuilder>::Type,
            >;
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for FungiblesRef {
        #[inline]
        fn clone(&self) -> FungiblesRef {
            FungiblesRef {
                inner: ::core::clone::Clone::clone(&self.inner),
            }
        }
    }
    const _: () = {
        impl ::ink::storage::traits::StorageLayout for FungiblesRef {
            fn layout(
                __key: &::ink::primitives::Key,
            ) -> ::ink::metadata::layout::Layout {
                ::ink::metadata::layout::Layout::Struct(
                    ::ink::metadata::layout::StructLayout::new(
                        "FungiblesRef",
                        [
                            ::ink::metadata::layout::FieldLayout::new(
                                "inner",
                                <<Fungibles as ::ink::codegen::ContractCallBuilder>::Type as ::ink::storage::traits::StorageLayout>::layout(
                                    __key,
                                ),
                            ),
                        ],
                    ),
                )
            }
        }
    };
    const _: () = {
        impl ::ink::env::ContractReference for Fungibles {
            type Type = FungiblesRef;
        }
        impl ::ink::env::call::ConstructorReturnType<FungiblesRef> for Fungibles {
            type Output = FungiblesRef;
            type Error = ();
            fn ok(value: FungiblesRef) -> Self::Output {
                value
            }
        }
        impl<E> ::ink::env::call::ConstructorReturnType<FungiblesRef>
        for ::core::result::Result<Fungibles, E>
        where
            E: ::ink::scale::Decode,
        {
            const IS_RESULT: bool = true;
            type Output = ::core::result::Result<FungiblesRef, E>;
            type Error = E;
            fn ok(value: FungiblesRef) -> Self::Output {
                ::core::result::Result::Ok(value)
            }
            fn err(err: Self::Error) -> ::core::option::Option<Self::Output> {
                ::core::option::Option::Some(::core::result::Result::Err(err))
            }
        }
        impl ::ink::env::ContractEnv for FungiblesRef {
            type Env = <Fungibles as ::ink::env::ContractEnv>::Env;
        }
    };
    impl FungiblesRef {
        #[inline]
        #[allow(clippy::type_complexity)]
        pub fn new() -> ::ink::env::call::CreateBuilder<
            Environment,
            Self,
            ::ink::env::call::utils::Unset<Hash>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::LimitParamsV2<
                    <Fungibles as ::ink::env::ContractEnv>::Env,
                >,
            >,
            ::ink::env::call::utils::Unset<Balance>,
            ::ink::env::call::utils::Set<
                ::ink::env::call::ExecutionInput<
                    ::ink::env::call::utils::EmptyArgumentList,
                >,
            >,
            ::ink::env::call::utils::Unset<::ink::env::call::state::Salt>,
            ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<Self>>,
        > {
            ::ink::env::call::build_create::<Self>()
                .exec_input(
                    ::ink::env::call::ExecutionInput::new(
                        ::ink::env::call::Selector::new([
                            0x9B_u8,
                            0xAE_u8,
                            0x9D_u8,
                            0x5E_u8,
                        ]),
                    ),
                )
                .returns::<Self>()
        }
        #[inline]
        pub fn total_supply(&self, id: AssetId) -> Result<Balance> {
            self.try_total_supply(id)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "total_supply",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn try_total_supply(
            &self,
            id: AssetId,
        ) -> ::ink::MessageResult<Result<Balance>> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .total_supply(id)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "total_supply",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn balance_of(&self, id: AssetId, owner: AccountId32) -> Result<Balance> {
            self.try_balance_of(id, owner)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "balance_of",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn try_balance_of(
            &self,
            id: AssetId,
            owner: AccountId32,
        ) -> ::ink::MessageResult<Result<Balance>> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .balance_of(id, owner)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "balance_of",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn allowance(
            &self,
            id: AssetId,
            owner: AccountId32,
            spender: AccountId32,
        ) -> Result<Balance> {
            self.try_allowance(id, owner, spender)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "allowance",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn try_allowance(
            &self,
            id: AssetId,
            owner: AccountId32,
            spender: AccountId32,
        ) -> ::ink::MessageResult<Result<Balance>> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .allowance(id, owner, spender)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "allowance",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
            self.try_asset_exists(id)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "asset_exists",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn try_asset_exists(
            &self,
            id: AssetId,
        ) -> ::ink::MessageResult<Result<bool>> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .asset_exists(id)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "asset_exists",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn mint_asset(
            &self,
            id: u32,
            beneficiary: AccountId32,
            amount: Balance,
        ) -> Result<()> {
            self.try_mint_asset(id, beneficiary, amount)
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "mint_asset",
                            error,
                        ),
                    );
                })
        }
        #[inline]
        pub fn try_mint_asset(
            &self,
            id: u32,
            beneficiary: AccountId32,
            amount: Balance,
        ) -> ::ink::MessageResult<Result<()>> {
            <Self as ::ink::codegen::TraitCallBuilder>::call(self)
                .mint_asset(id, beneficiary, amount)
                .try_invoke()
                .unwrap_or_else(|error| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "encountered error while calling {0}::{1}: {2:?}",
                            "Fungibles",
                            "mint_asset",
                            error,
                        ),
                    );
                })
        }
    }
    const _: () = {
        impl ::ink::codegen::TraitCallBuilder for FungiblesRef {
            type Builder = <Fungibles as ::ink::codegen::ContractCallBuilder>::Type;
            #[inline]
            fn call(&self) -> &Self::Builder {
                &self.inner
            }
            #[inline]
            fn call_mut(&mut self) -> &mut Self::Builder {
                &mut self.inner
            }
        }
    };
    impl ::ink::env::call::FromAccountId<Environment> for FungiblesRef {
        #[inline]
        fn from_account_id(account_id: AccountId) -> Self {
            Self {
                inner: <<Fungibles as ::ink::codegen::ContractCallBuilder>::Type as ::ink::env::call::FromAccountId<
                    Environment,
                >>::from_account_id(account_id),
            }
        }
    }
    impl ::ink::ToAccountId<Environment> for FungiblesRef {
        #[inline]
        fn to_account_id(&self) -> AccountId {
            <<Fungibles as ::ink::codegen::ContractCallBuilder>::Type as ::ink::ToAccountId<
                Environment,
            >>::to_account_id(&self.inner)
        }
    }
    impl ::core::convert::AsRef<AccountId> for FungiblesRef {
        fn as_ref(&self) -> &AccountId {
            <_ as ::core::convert::AsRef<AccountId>>::as_ref(&self.inner)
        }
    }
    impl ::core::convert::AsMut<AccountId> for FungiblesRef {
        fn as_mut(&mut self) -> &mut AccountId {
            <_ as ::core::convert::AsMut<AccountId>>::as_mut(&mut self.inner)
        }
    }
    #[cfg(feature = "std")]
    #[cfg(not(feature = "ink-as-dependency"))]
    const _: () = {
        #[no_mangle]
        pub fn __ink_generate_metadata() -> ::ink::metadata::InkProject {
            let layout = ::ink::metadata::layout::Layout::Root(
                ::ink::metadata::layout::RootLayout::new(
                    <::ink::metadata::layout::LayoutKey as ::core::convert::From<
                        ::ink::primitives::Key,
                    >>::from(<Fungibles as ::ink::storage::traits::StorageKey>::KEY),
                    <Fungibles as ::ink::storage::traits::StorageLayout>::layout(
                        &<Fungibles as ::ink::storage::traits::StorageKey>::KEY,
                    ),
                    ::ink::scale_info::meta_type::<Fungibles>(),
                ),
            );
            ::ink::metadata::layout::ValidateLayout::validate(&layout)
                .unwrap_or_else(|error| {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!("metadata ink! generation failed: {0}", error),
                        );
                    }
                });
            ::ink::metadata::InkProject::new(
                layout,
                ::ink::metadata::ContractSpec::new()
                    .constructors([
                        ::ink::metadata::ConstructorSpec::from_label("new")
                            .selector([0x9B_u8, 0xAE_u8, 0x9D_u8, 0x5E_u8])
                            .args([])
                            .payable(true)
                            .default(false)
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    if <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                                        2611912030u32,
                                    >>::IS_RESULT {
                                        ::ink::metadata::TypeSpec::with_name_str::<
                                            ::ink::ConstructorResult<
                                                ::core::result::Result<
                                                    (),
                                                    <Fungibles as ::ink::reflect::DispatchableConstructorInfo<
                                                        2611912030u32,
                                                    >>::Error,
                                                >,
                                            >,
                                        >("ink_primitives::ConstructorResult")
                                    } else {
                                        ::ink::metadata::TypeSpec::with_name_str::<
                                            ::ink::ConstructorResult<()>,
                                        >("ink_primitives::ConstructorResult")
                                    },
                                ),
                            )
                            .docs([])
                            .done(),
                    ])
                    .messages([
                        ::ink::metadata::MessageSpec::from_label("total_supply")
                            .selector([0xDB_u8, 0x63_u8, 0x75_u8, 0xA8_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("id")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AssetId,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AssetId"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<Result<Balance>>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(false)
                            .payable(false)
                            .default(false)
                            .docs([])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("balance_of")
                            .selector([0x0F_u8, 0x75_u8, 0x5A_u8, 0x56_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("id")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AssetId,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AssetId"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                                ::ink::metadata::MessageParamSpec::new("owner")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AccountId32,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AccountId32"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<Result<Balance>>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(false)
                            .payable(false)
                            .default(false)
                            .docs([])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("allowance")
                            .selector([0x6A_u8, 0x00_u8, 0x16_u8, 0x5E_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("id")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AssetId,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AssetId"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                                ::ink::metadata::MessageParamSpec::new("owner")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AccountId32,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AccountId32"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                                ::ink::metadata::MessageParamSpec::new("spender")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AccountId32,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AccountId32"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<Result<Balance>>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(false)
                            .payable(false)
                            .default(false)
                            .docs([])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("asset_exists")
                            .selector([0xAA_u8, 0x6B_u8, 0x65_u8, 0xDB_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("id")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AssetId,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AssetId"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<Result<bool>>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(false)
                            .payable(false)
                            .default(false)
                            .docs([])
                            .done(),
                        ::ink::metadata::MessageSpec::from_label("mint_asset")
                            .selector([0x1F_u8, 0x8E_u8, 0x8E_u8, 0x22_u8])
                            .args([
                                ::ink::metadata::MessageParamSpec::new("id")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            u32,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["u32"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                                ::ink::metadata::MessageParamSpec::new("beneficiary")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            AccountId32,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["AccountId32"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                                ::ink::metadata::MessageParamSpec::new("amount")
                                    .of_type(
                                        ::ink::metadata::TypeSpec::with_name_segs::<
                                            Balance,
                                            _,
                                        >(
                                            ::core::iter::Iterator::map(
                                                ::core::iter::IntoIterator::into_iter(["Balance"]),
                                                ::core::convert::AsRef::as_ref,
                                            ),
                                        ),
                                    )
                                    .done(),
                            ])
                            .returns(
                                ::ink::metadata::ReturnTypeSpec::new(
                                    ::ink::metadata::TypeSpec::with_name_segs::<
                                        ::ink::MessageResult<Result<()>>,
                                        _,
                                    >(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([
                                                "ink",
                                                "MessageResult",
                                            ]),
                                            ::core::convert::AsRef::as_ref,
                                        ),
                                    ),
                                ),
                            )
                            .mutates(false)
                            .payable(false)
                            .default(false)
                            .docs([])
                            .done(),
                    ])
                    .collect_events()
                    .docs([])
                    .lang_error(
                        ::ink::metadata::TypeSpec::with_name_segs::<
                            ::ink::LangError,
                            _,
                        >(
                            ::core::iter::Iterator::map(
                                ::core::iter::IntoIterator::into_iter(["ink", "LangError"]),
                                ::core::convert::AsRef::as_ref,
                            ),
                        ),
                    )
                    .environment(
                        ::ink::metadata::EnvironmentSpec::new()
                            .account_id(
                                ::ink::metadata::TypeSpec::with_name_segs::<
                                    AccountId,
                                    _,
                                >(
                                    ::core::iter::Iterator::map(
                                        ::core::iter::IntoIterator::into_iter(["AccountId"]),
                                        ::core::convert::AsRef::as_ref,
                                    ),
                                ),
                            )
                            .balance(
                                ::ink::metadata::TypeSpec::with_name_segs::<
                                    Balance,
                                    _,
                                >(
                                    ::core::iter::Iterator::map(
                                        ::core::iter::IntoIterator::into_iter(["Balance"]),
                                        ::core::convert::AsRef::as_ref,
                                    ),
                                ),
                            )
                            .hash(
                                ::ink::metadata::TypeSpec::with_name_segs::<
                                    Hash,
                                    _,
                                >(
                                    ::core::iter::Iterator::map(
                                        ::core::iter::IntoIterator::into_iter(["Hash"]),
                                        ::core::convert::AsRef::as_ref,
                                    ),
                                ),
                            )
                            .timestamp(
                                ::ink::metadata::TypeSpec::with_name_segs::<
                                    Timestamp,
                                    _,
                                >(
                                    ::core::iter::Iterator::map(
                                        ::core::iter::IntoIterator::into_iter(["Timestamp"]),
                                        ::core::convert::AsRef::as_ref,
                                    ),
                                ),
                            )
                            .block_number(
                                ::ink::metadata::TypeSpec::with_name_segs::<
                                    BlockNumber,
                                    _,
                                >(
                                    ::core::iter::Iterator::map(
                                        ::core::iter::IntoIterator::into_iter(["BlockNumber"]),
                                        ::core::convert::AsRef::as_ref,
                                    ),
                                ),
                            )
                            .chain_extension(
                                ::ink::metadata::TypeSpec::with_name_segs::<
                                    ChainExtension,
                                    _,
                                >(
                                    ::core::iter::Iterator::map(
                                        ::core::iter::IntoIterator::into_iter(["ChainExtension"]),
                                        ::core::convert::AsRef::as_ref,
                                    ),
                                ),
                            )
                            .max_event_topics(MAX_EVENT_TOPICS)
                            .static_buffer_size(::ink::env::BUFFER_SIZE)
                            .done(),
                    )
                    .done(),
            )
        }
    };
    use super::*;
}
