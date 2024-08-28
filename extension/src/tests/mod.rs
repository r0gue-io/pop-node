use crate::mock::{Test as Runtime, *};
use codec::Decode;
use frame_system::Call;
use sp_runtime::{BuildStorage, DispatchError};
use utils::{call, instantiate, ALICE, GAS_LIMIT, INIT_AMOUNT, INVALID_FUNC_ID};

mod encoding;
mod utils;

#[test]
fn dispatch_call_works() {
	new_test_ext().execute_with(|| {
		let contract = instantiate();

		let call = call(
			contract,
			DispatchCallFuncId::get(),
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);

		let return_value = call.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[..]).unwrap();
		assert!(decoded.unwrap().is_empty());

		assert!(call.events.unwrap().iter().any(|e| matches!(e.event,
				RuntimeEvent::System(frame_system::Event::<Test>::Remarked { sender, .. })
					if sender == contract)));
	});
}

#[test]
fn invalid_func_id_fails() {
	new_test_ext().execute_with(|| {
		let contract = instantiate();

		let call = call(contract, INVALID_FUNC_ID, (), GAS_LIMIT);
		let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
		// TODO: assess whether this error should be passed through the error converter - i.e. is this error type considered 'stable'?
		assert_eq!(call.result, Err(expected))
	});
}

fn new_test_ext() -> sp_io::TestExternalities {
	let _ = env_logger::try_init();

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, INIT_AMOUNT)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
