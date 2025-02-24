use super::*;

// Get the last event from pallet contracts.
pub(super) fn last_contract_event() -> Vec<u8> {
	let events = System::read_events_for_pallet::<pallet_contracts::Event<Runtime>>();
	let contract_events = events
		.iter()
		.filter_map(|event| match event {
			pallet_contracts::Event::<Runtime>::ContractEmitted { data, .. } =>
				Some(data.as_slice()),
			_ => None,
		})
		.collect::<Vec<&[u8]>>();
	contract_events.last().unwrap().to_vec()
}

// Decodes a byte slice into an `AccountId` as defined in `primitives`.
//
// This is used to resolve type mismatches between the `AccountId` in the integration tests and the
// contract environment.
pub(super) fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
	pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
}

pub(super) fn do_bare_call(function: &str, addr: &AccountId32, params: Vec<u8>) -> ExecReturnValue {
	let function = function_selector(function);
	let params = [function, params].concat();
	bare_call(addr.clone(), params, 0).expect("should work")
}

pub(super) fn decoded<T: Decode>(result: ExecReturnValue) -> Result<T, ExecReturnValue> {
	<T>::decode(&mut &result.data[1..]).map_err(|_| result)
}
