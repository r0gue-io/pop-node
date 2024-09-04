use crate::{mock::*, ContractWeights, Weight};
use pallet_contracts::WeightInfo;

// Weight charged for calling into the runtime from a contract.
pub(crate) fn overhead_weight(input_len: u32) -> Weight {
	ContractWeights::<Test>::seal_debug_message(input_len)
}

// Weight charged for reading function call input from buffer.
pub(crate) fn read_from_buffer_weight(input_len: u32) -> Weight {
	ContractWeights::<Test>::seal_return(input_len)
}

// Weight charged for writing to contract memory.
pub(crate) fn write_to_contract_weight(len: u32) -> Weight {
	ContractWeights::<Test>::seal_input(len)
}

// Weight charged after decoding failed.
pub(crate) fn decoding_failed_weight(input_len: u32) -> Weight {
	overhead_weight(input_len) + read_from_buffer_weight(input_len)
}
