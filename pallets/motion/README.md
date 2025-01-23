# Motion Pallet

`pallet-motion` enables councils (`pallet-collective`) to make root-origin calls by providing three configurable origins:
- `SimpleMajority` (1/2)
- `SuperMajority` (2/3)
- `Unanimous` (1/1)

It is composed of three associated extrinsics, one for each origin:
- `simple_majority`
    - Dispatches the call if and only if the number of votes is greater than `SimpleMajority`.
- `super_majority`
    - Dispatches the call if and only if the number of votes is greater than or equal to `SuperMajority`.
- `unanimous`
    - Dispatches the call if and only if all collective members have voted yes.


## Configuration

You can configure `pallet-motion` in combination with `pallet-collective` the following way:

```rust
type CouncilCollective = pallet_collective::Instance1;
impl pallet_motion::Config for Runtime {
	// ---
	type SimpleMajorityOrigin =
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>;
	type SuperMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type UnanimousOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
}
```
