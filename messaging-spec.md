# Maximus Messaging Monilith!
The messaging feature enables smart contracts on pop-net to seamlessly interact with external chains using XCM and GMP, it will also allow more messaging protocols to be added, like GMP. By providing a flexible and efficient messaging framework, it expands multichain use cases while giving developers full control over request handling and response processing. This system enhances interoperability and significantly improves the developer experience.
Key capabilities include:

- Enabling cross-chain message passing for smart contracts via XCM and ISMP.
- Supporting asynchronous callbacks, allowing contracts to handle responses efficiently.
- Providing granular control over message composition, dispatch, and response handling, within the framework of a given messaging protocol.
- Ensuring that weight costs are predictable (at least through dry runs) and charged for the full execution flow from message dispatch to callback execution.

This specification focuses on core messaging functionality and does not include high-level abstractions, prebuilt integrations, or advanced reliability mechanisms beyond the guarantees provided by XCM and ISMP.

## Non-functional Deliverables
The "API" stands for the api the smart contracts have access to.

- **Stable:** contracts are generally immutable and therefore need a stable interface, especially as the runtime may be evolving over time.
- **Future Proof:** technologies improve, so the API implementation should be adaptable to new approaches without affecting the interface. The API implementation should also select the optimal approach to realise a use case from a range of possible technical implementations.
- **Versioned:** not every future technology can be retro-fitted, so the API will be versioned.
- **Simple:** we want to make the use of the API by smart contract developers as easy to use as possible. A one-to-one mapping to pallet functionality would echo complexity up the stack. Pop API strives to shield dApp developers from this complexity.
- **Storage Efficient**: The contracts size impacts the cost to deploy.
- **Test Coverage**: Unit test coverage for each extrinsic at least 80%

## Functional Deliverables
- A contract can send a message to the pop-net runtime.
- A contract on pop-net can send an ISMP GET message to a foreign chain.
- A contract on pop-net can send an ISMP POST message to a foreign chain.
- A contract on pop-net can send an arbitrary XCM-based message to a foreign chain.
- A contract on pop-net can register a callback for execution on message response, regardless of message type.
- A contract on pop-net can query a response regardless of message type.
- A contract is able to remove request data if they have permission.
- A contract using Ismp is forced to use signed requests.
- A contract caller will receive any gas that is prepaid and unused.

There are 3 main areas to consider.
1. Runtime: Responsible for recording message state and sending messages, index versioning (errors, runtime calls) and initiating callbacks.
2. Chain extension: Acting as the communication layer between the contract and the runtime. Responsible for dispatching runtime calls and since it has access to the contract environment it is also responsible for charging weight appropriately.
3. Contract api: A collection of ink abstractions a contract developer can use to easily access exposed runtime features.

## Non-deliverables
- A high-level abstraction layer for developers (this version focuses on raw ISMP/XCM capabilities)
- Prebuilt integrations with specific parachains.
- Automated retries or reliability mechanisms beyond ISMP/XCM’s existing guarantees.
- Supporting ISMP data processing for RLP encoded contracts.
- Continuously running relayer (e.g. in pop-node that automatically relays messages).
- Adding other protocols beyond ISMP and XCM.
- Drink! e2e testing.
- Weight calculation tool for developers.
- Callbacks that have the ability to reference code to execute other than what is defined within the contract.
- The ability to recieve requests from external chains

## Architecture
todo!();



## Considerations: 
### Weight + Benchmarking
#### Weight Flow inspired from POC:
0. Contract is called with "to be charged" weight included.
1. Weight charged in contract by pallet-revive metering.
2. Weight charged for outgoing dispatchable.
3. Weight is charged for response hook.
4. Weight is charged for a standard call to `CallbackExecutor`.
5. Weight is charged for developer defined callback weight.

Since the `CallbackExecutor` is dependant on a runtime implementation, the benchmarking cannot accurately estimate the calling of a callback into contract, this must be defined by the runtime implementation.

The chain extension is responsible for charging the weight. It (chainextension) charges the weight of **The dispatchable it is calling** therefore, the weight of the extrinsic that sends messages must be the sum of 2, 3, 4 and 5.

It is the responsibility of the runtime developer inplementing CallbackExecutor to return the correct weight of execution.

Furthermore we must ensure that on_response (which is the hook that will be charged for responses ahead of time) is always the most expensive case with a sanity test.

#### On charging for future execution on the current block
Consider the argument.

1. When the chain extension invokes a dispatchable function, the weight of the associated callback is already included in its calculation.
2. The chain extension charges the contract environment for the full weight of the dispatchable call.
3. This weight contributes to filling the current block (Block 1), meaning some of the available blockspace is consumed.
4. However, the callback itself is scheduled for execution in a future block (Block 2).
5. When Block 2 arrives, it must also allocate space to execute the callback, consuming additional blockspace.

Conclusion: Since blockspace is consumed both when the dispatchable is initially called and when the callback executes, we effectively pay for the callback's weight twice, once in Block 1 and again in Block 2.


A solution to this is deducting the weight of the callback + callback execution in block 1.
Since the weight is not actually being used (since we are precharging), this can be considered safe (but a little hacky).
We can acheive this using `TransactionExtension` specifically the `StorageWeightReclaim` type.

#### On XCMs handling of weight for responses.

TODO!("Does XCM charge the weight of the runtime call when it handles a response?")


### Response Hooks
For ismp we have the `IsmpModule` trait which is called when ismp-pallet receives a response, timeout or accepts a request. 
We also have the corrosponding `IsmpModuleWeight` which we can use for the pre execution charge.

For xcm its a little different, we must define an extrinsic that xcm-pallet calls as its notify parameter. 
Then we benchmark the response extrinsic like normal.

Both of these are dependant on the CallbackExecutor.

### XCM and Queries
We need to tell pallet-xcm to expect a new query regardless of which method we use to generate message ids. 
For this we can use: 
```rust
	/// Attempt to create a new query ID and register it as a query that is yet to respond, and
	/// which will call a dispatchable when a response happens.
	pub fn new_notify_query(
		responder: impl Into<Location>,
		notify: impl Into<<T as Config>::RuntimeCall>,
		timeout: BlockNumberFor<T>,
		match_querier: impl Into<Location>,
	) -> u64 {
        // snip 
    }
```
Unfortunately the QueryHandler trait doesnt contain the notify parameter hence we have to use a wrapper trait: 
```rust
pub trait NotifyQueryHandler<T: Config> {
	/// Attempt to create a new query ID and register it as a query that is yet to respond, and
	/// which will call a dispatchable when a response happens.
	fn new_notify_query(
		responder: impl Into<Location>,
		notify: Call<T>,
		timeout: BlockNumberOf<T>,
		match_querier: impl Into<Location>,
	) -> u64;
}
```

This returns us a XCM query_id, not to be confused with our message_id, which we can use to match to a request and therefore the registered call back.
When pallet-xcm recieves the response it will decode the runtime call and call our extrinsic for response handling.

### ISMP and Queries
We can use the `IsmpDispatcher` to dispatch all outgoing calls.

### Message IDs
Message ids are used as a key for outgoing requests, they can be queried to find the status of a message.
A message id can either be specified by the runtime or the contract.

If the message id is specified by the runtime, the runtime will have to return that data to the contract to emit or emit an event itself. The event emitted must contain data that can relate the request to a response.

If the message id is specified by the contract, the contract developer has full control over the shape of the id and can emit a reference that can be used to link a request with a response. This must be considered by the contract developer therefore, one can create a library for ink developers (both for the contract and for the app side) that allows contract developers to easily match responses and requests.

the latter will by implemented, the actual implementation is simpler and allows full flexibility for developers.

TLDR: 
- Allow developers to specify a message ID.
- Use a cryptographically secure hasher.

### RequestDeposits
For each outgoing request we will take a request deposit from the caller.
This will be calculated by the request encoded len  *  byte fee. (TODO: more clarification)


### Extrinsics
See "Weight and Benchmarking" for a walkthrough of weight charging.

```rust 
        /// Send an ismp get message and optionally specify a callback.
		#[pallet::call_index(1)]
		#[pallet::weight(
			T::WeightInfo::ismp_get() + 
            T::IsmpModuleWeights::on_response() + 
            callback.map(|cb| cb.weight + T::CallbackExecutor::execution_weight()).unwrap_or_default()
		)]
		pub fn ismp_get(
			origin: OriginFor<T>,
			id: MessageId,
			message: ismp::Get<T>,
			fee: BalanceOf<T>,
			callback: Option<Callback>,
		) -> DispatchResult {
            // snip
        }
```

```rust
        /// Send an ismp post message and optionally specify a callback.
		#[pallet::call_index(2)]
		#[pallet::weight(	
            T::WeightInfo::ismp_post() + 
            T::IsmpModuleWeights::on_response() + 
            callback.map(|cb| cb.weight + T::CallbackExecutor::execution_weight()).unwrap_or_default()
		)]
		pub fn ismp_post(
			origin: OriginFor<T>,
			id: MessageId,
			message: ismp::Post<T>,
			fee: BalanceOf<T>,
			callback: Option<Callback>,
		) -> DispatchResult {
            // snip
        }
```

```rust
        /// Send an xcm query and optionally specify a callback.
	    #[pallet::call_index(3)]
		#[pallet::weight(
            T::WeightInfo::xcm_new_query() +
            T::WeightInfo::xcm_response() +
            callback.map(|cb| cb.weight + T::CallbackExecutor::execution_weight()).unwrap_or_default()
		)]
		pub fn xcm_new_query(
			origin: OriginFor<T>,
			id: u64,
			responder: Location,
			timeout: BlockNumberOf<T>,
			callback: Option<Callback>,
		) -> DispatchResult {
            // snip
        }
```

```rust
        /// Handle an xcm response.
        /// Can only be called with via a Location origin.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::xcm_response())] 
		pub fn xcm_response(
			origin: OriginFor<T>,
			query_id: QueryId,
			response: Response,
		) -> DispatchResult {
            // snip
        }
```

```rust 
        /// Remove a selection of messages.
	    #[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove(messages.len()))]
		pub fn remove(
			origin: OriginFor<T>,
			messages: BoundedVec<MessageId, T::MaxMessageRemovals>,
		) -> DispatchResult {
            // snip
        }
```



### Pop-api:
TODO!("Define what the pop-api will look like, i doubt much will change from the current POC implementation.")

## Possible Future Work
#### How much weight should a caller send with the contract invocation?
Since the weight of the entire flow must be charged ahead of time, we leave the developer with the hassle of having to estimate the weight it sends on contract invocation.
This is poor developer experience as a developer will have to dry run the most expensive case ahead of time to calculate the expected weight. Even more so, if the pallet code changes and the weight of any of the extrinsics change, the contract developer may have to recalculate.

Furthermore with contract execution being coupled to mutable runtime extrinsic weight we cannot fallback on versioning to solve this. If the weight of an extrinsic increases (V1) we cannot then charge the contract the weight of the lesser (V0). 

Some ideas:
Provide an api for app developers that returns an estimated cost for a specific set of instructions (is this much better than a dry run lol?)
im out of ideas ngl gonna sleep on it.

#### Callbacks that dont reference a selector within the origin contract.
I am a contract developer and i want to make attempt a funglibles::transfer call in my callback. 
It would certainly be cheaper to do the call directly in the runtime (where callbacks are executed) instead of hopping from the runtime env to the contract env and back to the runtime env to make this call.

#### Defining the block height of a GET request
The ISMP protocol requires us to provide a block height to query from.
A runtime developer implementing pallet-ismp can use the following storage item to query data: 
```rust
    /// The latest verified height for a state machine
    #[pallet::storage]
    #[pallet::getter(fn latest_state_height)]
    pub type LatestStateMachineHeight<T: Config> =
        StorageMap<_, Blake2_128Concat, StateMachineId, u64, ValueQuery>;
```
Hence we could have an optional block height in the parameters and if empty then populate using runtime storage.

Alternatively an app can call an rpc to retrieve the latest finalised block height and populate the ink message accordingly.
Since the spec will focus mainly on delivering raw ISMP capabilites, I think for this iteration, handing this responisbility to the developer is a suitable solution.

#### Receiving requests from external chains via ismp
This is not in scope for the initial implementation. the `on_accept` hook of the `IsmpModule` is responsible for such things.
What post operations should we allow? 
What get queries should we allow? 

#### Maintaining multiple signatures of the same type
An example being the Callback type, both defined on the contract side and in the pallet code.
Having multiple signatures for the same type prevents us from coupling what the contract needs to know and what the runtime needs to know. This is actually an advantage. I used to think it bad ngl. I think however this can be improved to have a single place where these signatures are held. 
Think contract_api_types and runtime_api_types. This way we can easily view the relationship between types.


# Pallet Tasks  
## ISMP  
### Implementation  
- [ ] Implement `IsmpModule::on_accept`  
- [ ] Implement `IsmpModule::Timeout::Response`  

### Benchmarks  
- [ ] Benchmark `IsmpModuleWeight::on_accept`  
- [ ] Benchmark `IsmpModuleWeight::on_timeout`  
- [ ] Benchmark `IsmpModuleWeight::on_response`  
- [ ] Benchmark `ismp_get` extrinsic  
- [ ] Benchmark `ismp_post` extrinsic  

### Audits & Improvements  
- [ ] Self-audit deposits for requests  
- [ ] Ensure callback request can be queried on callback revert 
- [ ] Ensure that ISMP is using signed responses and requests.
- [ ] update to latest version of ismp-substrate.

### Testing  
- [ ] Mock environment for the messaging pallet  
- [ ] Write unit tests for `ismp_get` extrinsic  
- [ ] Write unit tests for `ismp_post` extrinsic  

---

## XCM  
### Benchmarks  
- [ ] Benchmark `xcm_new_query` extrinsic  
- [ ] Benchmark `xcm_response` extrinsic  

### Testing  
- [ ] Write unit tests for `xcm_new_query` extrinsic  
- [ ] Write unit tests for `xcm_response` extrinsic  
- [ ] Test `xcm_new_query` to `xcm_response` flow  

### Improvements
- [ ] Default implementation for NotifyQueryHandler


---

## General Improvements  
### Benchmarks & Performance  
- [ ] Benchmark `remove` extrinsic  
- [ ] Correct weight for the CallbackExecutor  
- [ ] Use a transaction extension to reclaim weight for a callback on the requesting block.

### Code Quality & Refactoring  
- [ ] Audit `calculate_deposits`  
- [ ] Refactor deposits (code is scattered and hard to follow) 
- [ ] Rename CallbackT to CallbackExecutor 

### Testing  
- [ ] Write unit tests for `remove` extrinsic  
- [ ] Write runtime configuration tests  
- [ ] Ensure that on_response is the most expensive case via sanity test.

### Documentation  
- [ ] Improve documentation (clarity, completeness, structure)  
- [ ] Create a **Weight Charging Diagram** and corresponding documentation  
- [ ] Write **Version Control Documentation**  