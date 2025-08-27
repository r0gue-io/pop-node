# Messaging API

The messaging API offers a streamlined interface for cross-chain interactions. The goal is to provide a simplified API that unlocks the power of Polkadot for contracts.

## ISMP
The following diagram illustrates the flow of the ISMP implementation:

```mermaid
flowchart TB
    A["Contract"] -- Ismp::get/post --> B["messaging::ISMP Precompile"]
    B -- transports::ismp::get/post --> n4["Take Deposit"]
    n1["runtime::Router"] -- ISMPModule::on_response --> n2["messaging::ISMP Module"]
    n2 --> n3["Callback?"]
    D["Relayer"] -- "pallet-ismp::handle(messages)" --> n1
    n3 -- "<span style=padding-left:><span style=padding-left:>onGetResponse/onPostReponse</span></span>" --> A
    C["pallet-ismp::ISMP Dispatcher"] -.- D
    n4 -- IsmpDispatcher::dispatch_request --> C
    C --> n5["Persist Request/Message"]

    n4@{ shape: rounded}
    n1@{ shape: rect}
    n2@{ shape: rect}
    n3@{ shape: diam}
    D@{ shape: hex}
    n5@{ shape: rounded}
```

## XCM
The following diagram illustrates the flow of the XCM implementation:

```mermaid
flowchart TB
    A["Contract"] -- Xcm::newQuery --> B["messaging::XCM Precompile"]
    B -- transports::xcm::new_query --> n4["Take Deposit"]
    n1["xcm_executor"] -- Pallet::xcm_response --> n2["messaging::Pallet"]
    n2 --> n3["Callback?"]
    D["XCM"] -- </br> --> n1
    n3 -- onQueryResponse --> A
    C["pallet-xcm"] -.- D
    n4 -- Xcm::new_notify_query --> C
    C --> n5["Persist Request/Message"]
    A -- Xcm::send --> B
    B -- Pallet::send --> C

    n4@{ shape: rounded}
    n1@{ shape: rect}
    n2@{ shape: rect}
    n3@{ shape: diam}
    D@{ shape: hex}
    n5@{ shape: rounded}
```

## Weights

A description on how fees, blockspace and weights are handled can be found [here](weights.md).
