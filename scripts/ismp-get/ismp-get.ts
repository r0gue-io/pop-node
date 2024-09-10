import {ApiPromise, Keyring, WsProvider} from "@polkadot/api";
import {KeyringPair} from "@polkadot/keyring/types";
import {submitExtrinsic, transferRelayAssetToPara} from "./common";
import {
    IsmpRequest,
    Events,
    API_TYPES,
    CUSTOM_RPC,
    StateMachine,
} from "./types";

const AH_ID = 1000;
const POP_ID = 4001;

async function run() {
    console.log("starting up...");
    const pop_uri = "ws://127.0.0.1:9944";
    const paseo_uri = "ws://127.0.0.1:8833";
    const ah_uri = "ws://127.0.0.1:9977";

    const popApi = await ApiPromise.create({
        provider: new WsProvider(pop_uri),
        types: {...API_TYPES},
        rpc: CUSTOM_RPC,
    });
    const paseoApi = await ApiPromise.create({
        provider: new WsProvider(paseo_uri),
    });
    const ahApi = await ApiPromise.create({
        provider: new WsProvider(ah_uri),
    });

    // account to submit tx
    const keyring = new Keyring({type: "sr25519"});
    const alice = keyring.addFromUri("//Alice");
    const receiverKeypair = new Keyring();
    receiverKeypair.addFromAddress(alice.address);

    console.log("Setting up asset");

    // Needed for fee payment
    await transferRelayAssetToPara(10n ** 12n, POP_ID, paseoApi, alice);
    // add AH as
    await ismpAddParachain(alice, popApi);

    const latest_height = await popApi.query.ismp.latestStateMachineHeight({
        stateId: {Polkadot: AH_ID},
        consensusStateId: "PARA"
    });
    console.log("Latest Height: ", latest_height.toString());

    // parachain info pallet fetching para id
    let encoded_chain_b_id_storage_key =
        "0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f";
    const requestRecord = popApi.tx.ismpDemo.getRequest({
        paraId: AH_ID,
        height: latest_height.toString(),
        timeout: 100000,
        keys: [encoded_chain_b_id_storage_key],
    });
    let events: Events = {events: []};
    await submitExtrinsic(alice, requestRecord, {}, events);

    console.log("events: ", events);
    console.log("Event found: ", findEvent(events.events, "ismp", "Request").toHuman());
    const ismpRequest = findEvent(events.events, "ismp", "Request");
    const commitmentRequest = ismpRequest.event.data.commitment;
    console.log("Commitment: ", commitmentRequest.toHex());

    // Check the data on the Coretime chain:
    const ahId = await ahApi.query.parachainInfo.parachainId();

    console.log("AH ID: ", ahId.toString());

    // Respond to the ISMP get request:
    const request = await queryRequest(popApi, commitmentRequest.toString());
    console.log("Request: ");
    console.log(request);
    // request.get.source;
    await makeIsmpResponse(popApi, ahApi, request, alice.address);
}

// Function to convert the string to a StateMachine
function convertToStateMachine(input: string): StateMachine {
    const [network, value] = input.split("-");
    const numericValue = parseInt(value, 10);

    switch (network.toUpperCase()) {
        case "POLKADOT":
            return {Polkadot: numericValue};
        case "KUSAMA":
            return {Kusama: numericValue};
        // Add cases for other networks as needed
        default:
            throw new Error(`Unsupported network: ${network}`);
    }
}

function findEvent(events: any[], section: string, method: string): any {
    return events.find((event) => event.event.section == section && event.event.method == method);

}

async function ismpAddParachain(signer: KeyringPair, popApi: ApiPromise, para_id: number = AH_ID) {
    const addParaCall = popApi.tx.ismpParachain.addParachain([
        {id: para_id, slotDuration: 6000},
    ]);
    const sudoCall = popApi.tx.sudo.sudo(addParaCall);
    return submitExtrinsic(signer, sudoCall, {}, {events: []});
}

async function queryRequest(
    popApi: ApiPromise,
    commitment: string,
): Promise<IsmpRequest> {
    console.log("Here");
    const leafIndex = popApi.createType("LeafIndexQuery", {
        commitment,
    });
    console.log("LeafIndex: ", leafIndex.toHuman());
    const requests = await (popApi as any).rpc.ismp.queryRequests([
        leafIndex,
    ]);
    console.log("QueryRequest: " + requests);

    const request = requests.toJSON()[0] as IsmpRequest;
    if ("get" in request) {
        request.get.source = convertToStateMachine(`POLKADOT-${POP_ID}`);
        request.get.dest = convertToStateMachine(`POLKADOT-${AH_ID}`);
    }

    // We only requested a single request so we only get one in the response.
    return request;
}

async function makeIsmpResponse(
    popApi: ApiPromise,
    ahApi: ApiPromise,
    request: IsmpRequest,
    responderAddress: string,
): Promise<void> {
    console.log("makeIsmpResponse");

    const hashAt = (
        await ahApi.query.system.blockHash(Number(request.get.height))
    ).toString();
    console.log("hashAt: " + hashAt);
    const proofData = await ahApi.rpc.state.getReadProof(
        [request.get.keys[0]],
        hashAt,
    );
    console.log("proofData: " + proofData);

    const stateMachineProof = popApi.createType("StateMachineProof", {
        hasher: "Blake2",
        storage_proof: proofData.proof,
    });

    const substrateStateProof = popApi.createType("SubstrateStateProof", {
        StateProof: stateMachineProof,
    });

    const response = popApi.tx.ismp.handleUnsigned([
        {
            Response: {
                datagram: {
                    Request: [request],
                },
                proof: {
                    height: {
                        id: {
                            stateId: {
                                Polkadot: AH_ID,
                            },
                            consensusStateId: "PARA",
                        },
                        height: request.get.height.toString(),
                    },
                    proof: substrateStateProof.toHex(),
                },
                signer: responderAddress,
            },
        },
    ]);
    console.log("response: " + response);

    // unsigned transaction
    return new Promise((resolve, reject) => {
        const unsub = response.send(({status, isError}) => {
            console.log(`Current status is ${status}`);
            if (status.isInBlock) {
                console.log(`Transaction included at blockHash ${status.asInBlock}`);
            } else if (status.isFinalized) {
                console.log(
                    `Transaction finalized at blockHash ${status.asFinalized}`,
                );
                unsub.then();
                return resolve();
            } else if (isError) {
                console.log("Transaction error");
                unsub.then();
                return reject();
            }
        });
    });
}

run();