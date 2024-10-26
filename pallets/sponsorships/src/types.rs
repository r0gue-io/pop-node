use frame_support::traits::Currency;

use crate::Config;

/// AccountId alias
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
/// Balance alias
pub(crate) type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
