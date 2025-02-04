/// Implements encoding and decoding traits for a wrapper type that represents
/// bitflags. The wrapper type should contain a field of type `$size`, where
/// `$size` is an integer type (e.g., u8, u16, u32) that can represent the bitflags.
/// The `$bitflag_enum` type is the enumeration type that defines the individual bitflags.
///
/// This macro provides implementations for the following traits:
/// - `MaxEncodedLen`: Calculates the maximum encoded length for the wrapper type.
/// - `Encode`: Encodes the wrapper type using the provided encoding function.
/// - `Decode`: Decodes the wrapper type from the input.
macro_rules! impl_codec_bitflags {
	($wrapper:ty, $size:ty, $bitflag_enum:ty) => {
		impl ink::scale::MaxEncodedLen for $wrapper {
			fn max_encoded_len() -> usize {
				<$size>::max_encoded_len()
			}
		}
		impl ink::scale::Encode for $wrapper {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				self.0.bits().using_encoded(f)
			}
		}
		impl ink::scale::Decode for $wrapper {
			fn decode<I: ink::scale::Input>(
				input: &mut I,
			) -> ::core::result::Result<Self, ink::scale::Error> {
				let field = <$size>::decode(input)?;
				Ok(Self(BitFlags::from_bits(field as $size).map_err(|_| "invalid value")?))
			}
		}
	};
}
pub(crate) use impl_codec_bitflags;
