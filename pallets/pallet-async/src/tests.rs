use crate::{mock::*, Response, *};
use frame_support::{assert_noop, assert_ok};
use xcm::latest::Response::DispatchResult;

type BlockNumber = u64;

#[test]
fn register_query_works() {
	new_test_ext().execute_with(|| {
		let ismp_query = QueryType::Ismp;
		assert_eq!(GlobalQueryCounter::<Test>::get(), 0);
		// Dispatch a signed extrinsic.
		assert_ok!(Async::register_query(RuntimeOrigin::signed(1), ismp_query.clone()));
		assert_eq!(Queries::<Test>::get(0), Some((ismp_query, 1)));
		assert_eq!(GlobalQueryCounter::<Test>::get(), 1);

		let xcm_query = QueryType::Xcm(9);
		assert_eq!(GlobalQueryCounter::<Test>::get(), 1);
		// Dispatch a signed extrinsic.
		assert_ok!(Async::register_query(RuntimeOrigin::signed(1), xcm_query.clone()));
		assert_eq!(Queries::<Test>::get(1), Some((xcm_query, 1)));
		assert_eq!(GlobalQueryCounter::<Test>::get(), 2);
	});
}

#[test]
fn route_query_xcm_works() {
	new_test_ext().execute_with(|| {
		let ismp_query = QueryType::Ismp;

		let xcm_response_success = xcm::v4::Response::DispatchResult(MaybeErrorCode::Success);
		let expected_query_response: QueryResponseStatus<BlockNumber> =
			QueryResponseStatus::Ready { response: xcm_response_success.clone(), at: 0 };

		let responser_location = Location::parent();
		let xcm_query_id =
			TestQueryHandler::<Test, BlockNumber>::new_query(responser_location, 1, Here);
		let xcm_query = QueryType::Xcm(xcm_query_id);

		assert_eq!(Async::route_query(&xcm_query), Response::Xcm(QueryResponseStatus::NotFound));

		let xcm_context = XcmContext { origin: None, message_id: XcmHash::default(), topic: None };

		TestResponseHandler::on_response(
			&Location::parent(),
			xcm_query_id,
			None,
			DispatchResult(MaybeErrorCode::Success),
			Weight::zero(),
			&xcm_context,
		);

		let _global_query_id = GlobalQueryCounter::<Test>::get();
		assert_ok!(Async::register_query(RuntimeOrigin::signed(1), xcm_query.clone()));

		// TODO: responses are placeholders for ISMP
		let expected_ismp_response: Response<
			<TestQueryHandler<Test, BlockNumber> as QueryHandler>::BlockNumber,
		> = Response::Ismp(0);
		let expected_xcm_response: Response<
			<TestQueryHandler<Test, BlockNumber> as QueryHandler>::BlockNumber,
		> = Response::Xcm(expected_query_response);

		assert_eq!(Async::route_query(&ismp_query), expected_ismp_response);
		assert_eq!(Async::route_query(&xcm_query), expected_xcm_response);

		assert_eq!(response(xcm_query_id), Some(xcm_response_success));
	});
}

#[test]
fn take_response_works() {
	new_test_ext().execute_with(|| {
		// let ismp_query = QueryType::Ismp(1);
		//
		// let query_id = GlobalQueryCounter::<Test>::get();
		//
		// // register ISMP query
		// assert_ok!(Async::register_query(RuntimeOrigin::signed(1), ismp_query.clone()));
		//
		// // TODO: responses are placeholders until ISMP and XCM are integrated
		// let expected_ismp_response = Response { data: vec![0] };
		//
		// // only origin that registered the query can take the response
		// assert_noop!(
		// 	Async::take_response(RuntimeOrigin::signed(9999), query_id),
		// 	Error::<Test>::BadOrigin
		// );
		// assert_eq!(
		// 	Async::take_response(RuntimeOrigin::signed(1), query_id),
		// 	Ok(expected_ismp_response)
		// );
		// assert_eq!(Queries::<Test>::get(query_id), None);
	});
}
