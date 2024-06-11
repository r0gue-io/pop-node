use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_query_works() {
	new_test_ext().execute_with(|| {
		let ismp_query = QueryType::ISMP(1);
		assert_eq!(GlobalQueryCounter::<Test>::get(), 0);
		// Dispatch a signed extrinsic.
		assert_ok!(Async::register_query(RuntimeOrigin::signed(1), ismp_query.clone()));
		assert_eq!(Queries::<Test>::get(0), Some((ismp_query, 1)));
		assert_eq!(GlobalQueryCounter::<Test>::get(), 1);
	});
}

#[test]
fn route_query_works() {
	new_test_ext().execute_with(|| {
		let ismp_query = QueryType::ISMP(1);
		let xcm_query = QueryType::XCM(1);

		// TODO: responses are placeholders until ISMP and XCM are integrated
		let expected_ismp_response = Response { data: vec![0] };
		let expected_xcm_response = Response { data: vec![1] };

		assert_eq!(Async::route_query(ismp_query), expected_ismp_response);
		assert_eq!(Async::route_query(xcm_query), expected_xcm_response);
	});
}

#[test]
fn take_response_works() {
	new_test_ext().execute_with(|| {
		let ismp_query = QueryType::ISMP(1);

		let query_id = GlobalQueryCounter::<Test>::get();

		// register ISMP query
		assert_ok!(Async::register_query(RuntimeOrigin::signed(1), ismp_query.clone()));

		// TODO: responses are placeholders until ISMP and XCM are integrated
		let expected_ismp_response = Response { data: vec![0] };

		// only origin that registered the query can take the response
		assert_noop!(
			Async::take_response(RuntimeOrigin::signed(9999), query_id),
			Error::<Test>::BadOrigin
		);
		assert_eq!(
			Async::take_response(RuntimeOrigin::signed(1), query_id),
			Ok(expected_ismp_response)
		);
		assert_eq!(Queries::<Test>::get(query_id), None);
	});
}
