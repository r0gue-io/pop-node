import {ApiPromise, Keyring, WsProvider} from "@polkadot/api";
import {KeyringPair} from "@polkadot/keyring/types";
// import { RegionId } from "coretime-utils";
import {submitExtrinsic, transferRelayAssetToPara} from "../common";
import {CONFIG, CORE_COUNT, INITIAL_PRICE} from "../consts";
import {
    Get,
    IsmpRequest,
    REGIONX_API_TYPES,
    REGIONX_CUSTOM_RPC,
    StateMachine,
} from "./types";

const REGIONX_SOVEREIGN_ACCOUNT =
    "5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8";

async function run() {
    console.log("starting up...");
    const pop_uri = "ws://127.0.0.1:9944";
    const paseo_uri = "ws://127.0.0.1:8833";
    const ah_uri = "ws://127.0.0.1:9977";

    const popApi = await ApiPromise.create({
        provider: new WsProvider(pop_uri),
        types: {...REGIONX_API_TYPES},
        rpc: REGIONX_CUSTOM_RPC,
    });
    const rococoApi = await ApiPromise.create({
        provider: new WsProvider(paseo_uri),
    });
    const ahApi = await ApiPromise.create({
        provider: new WsProvider(ah_uri),
    });

    // account to submit tx
    const keyring = new Keyring({type: "sr25519"});
    const alice = keyring.addFromUri("//Alice");

    console.log("Setting up asset");
    // await setupRelayAsset(popApi, alice);

    // Needed for fee payment
    // The Coretime chain account by default has tokens for fee payment.
    await transferRelayAssetToPara(10n ** 12n, 4001, rococoApi, alice);

    // const txSetBalance = rococoApi.tx.balances.forceSetBalance(
    //   alice.address,
    //   10000 * UNIT,
    // );
    // await submitExtrinsic(alice, rococoApi.tx.sudo.sudo(txSetBalance), {});

    await ismpAddParachain(alice, popApi);

    const receiverKeypair = new Keyring();
    receiverKeypair.addFromAddress(alice.address);

    const feeAssetItem = 0;
    const weightLimit = "Unlimited";

    const latest_height = await popApi.query.ismp.latestStateMachineHeight({
        stateId: {Polkadot: 1000},
        consensusStateId: "PARA"
    });
    console.log("Latest Height: ", latest_height.toString());

    // const requestRecord = popApi.tx.regions.requestRegionRecord(regionId);
    // 	pub struct GetRequest {
    // 	/// Destination parachain
    // 	pub para_id: u32,
    // 	/// Height at which to read state
    // 	pub height: u32,
    // 	/// request timeout
    // 	pub timeout: u64,
    // 	/// Storage keys to read
    // 	pub keys: Vec<Vec<u8>>,
    // }
    // parachain info pallet fetching para id
    let encoded_chain_b_id_storage_key =
        "0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f";
    const requestRecord = popApi.tx.ismpDemo.getRequest({
        paraId: 1000,
        height: latest_height.toString(),
        timeout: 100000,
        keys: [encoded_chain_b_id_storage_key],
    });
    await submitExtrinsic(alice, requestRecord, {});

    const commitmentRequest = await popApi.query.ismpDemo.requests(0);
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

    // The record should be set after ISMP response:
    // regions = await popApi.query.regions.regions.entries();
    // region = regions[0][1].toHuman() as any;
    // assert(region.owner == alice.address);
    // assert.equal(region.record.Available.end, "66");
    // assert.equal(region.record.Available.paid, null);

    // regions = await popApi.query.regions.regions.entries();
    // assert.equal(regions.length, 0);

    // regions = await ahApi.query.broker.regions.entries();
    // assert.equal(regions.length, 1);
    // assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);
    // assert.equal((regions[0][1].toHuman() as any).owner, alice.address);
}

run();

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

async function ismpAddParachain(signer: KeyringPair, regionXApi: ApiPromise) {
    const addParaCall = regionXApi.tx.ismpParachain.addParachain([
        {id: 1000, slotDuration: 6000},
    ]);
    const sudoCall = regionXApi.tx.sudo.sudo(addParaCall);
    return submitExtrinsic(signer, sudoCall, {});
}

async function openHrmpChannel(
    signer: KeyringPair,
    relayApi: ApiPromise,
    senderParaId: number,
    recipientParaId: number,
) {
    const openHrmp = relayApi.tx.parasSudoWrapper.sudoEstablishHrmpChannel(
        senderParaId, // sender
        recipientParaId, // recipient
        8, // Max capacity
        102400, // Max message size
    );
    const sudoCall = relayApi.tx.sudo.sudo(openHrmp);

    return submitExtrinsic(signer, sudoCall, {});
}

async function configureBroker(
    coretimeApi: ApiPromise,
    signer: KeyringPair,
): Promise<void> {
    const configCall = coretimeApi.tx.broker.configure(CONFIG);
    const sudo = coretimeApi.tx.sudo.sudo(configCall);
    return submitExtrinsic(signer, sudo, {});
}

async function startSales(
    coretimeApi: ApiPromise,
    signer: KeyringPair,
): Promise<void> {
    const startSaleCall = coretimeApi.tx.broker.startSales(
        INITIAL_PRICE,
        CORE_COUNT,
    );
    const sudo = coretimeApi.tx.sudo.sudo(startSaleCall);
    return submitExtrinsic(signer, sudo, {});
}

async function queryRequest(
    regionxApi: ApiPromise,
    commitment: string,
): Promise<IsmpRequest> {
    console.log("Here");
    const leafIndex = regionxApi.createType("LeafIndexQuery", {
        commitment,
    });
    console.log("LeafIndex: ", leafIndex.toHuman());
    const requests = await (regionxApi as any).rpc.ismp.queryRequests([
        leafIndex,
    ]);
    console.log("QueryRequest: " + requests);

    const request = requests.toJSON()[0] as IsmpRequest;
    if ("get" in request) {
        request.get.source = convertToStateMachine("POLKADOT-4001");
        request.get.dest = convertToStateMachine("POLKADOT-1000");
    }

    // We only requested a single request so we only get one in the response.
    return request;
}

async function makeIsmpResponse(
    regionXApi: ApiPromise,
    coretimeApi: ApiPromise,
    request: IsmpRequest,
    responderAddress: string,
): Promise<void> {
    console.log("* makeIsmpResponse * ");
    if (isGetRequest(request)) {
        const hashAt = (
            await coretimeApi.query.system.blockHash(Number(request.get.height))
        ).toString();
        console.log("hashAt: " + hashAt);
        const proofData = await coretimeApi.rpc.state.getReadProof(
            [request.get.keys[0]],
            hashAt,
        );
        console.log("proofData: " + proofData);

        const stateMachineProof = regionXApi.createType("StateMachineProof", {
            hasher: "Blake2",
            storage_proof: proofData.proof,
        });

        const substrateStateProof = regionXApi.createType("SubstrateStateProof", {
            StateProof: stateMachineProof,
        });
        console.log("calling ismp handleUnsigned: ");
        const response = regionXApi.tx.ismp.handleUnsigned([
            {
                Response: {
                    datagram: {
                        Request: [request],
                    },
                    proof: {
                        height: {
                            id: {
                                stateId: {
                                    Polkadot: 1000,
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
    } else {
        new Error("Expected a Get request");
    }
}

const isGetRequest = (request: IsmpRequest): request is { get: Get } => {
    return true;
};

export {run};
