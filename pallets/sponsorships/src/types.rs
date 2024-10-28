use frame_support::traits::Currency;
use pallet_transaction_payment::OnChargeTransaction;

use crate::Config;

/// AccountId alias
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
/// Balance alias
pub(crate) type BalanceOf<T> = <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance;
/// Liquidity info type alias (imbalances).
pub(crate) type LiquidityInfoOf<T> =
<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo;
