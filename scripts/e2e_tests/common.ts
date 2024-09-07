import {ApiPromise, Keyring} from '@polkadot/api';
import {SignerOptions, SubmittableExtrinsic} from '@polkadot/api/types';
import {KeyringPair} from '@polkadot/keyring/types';
import {stringToU8a} from '@polkadot/util';
import {encodeAddress} from '@polkadot/util-crypto';

const RELAY_ASSET_ID = 1;

async function submitExtrinsic(
    signer: KeyringPair,
    call: SubmittableExtrinsic<'promise'>,
    options: Partial<SignerOptions>
): Promise<void> {
    return new Promise((resolve, reject) => {
        const unsub = call.signAndSend(signer, options, ({status, isError}) => {
            console.log(`Current status is ${status}`);
            if (status.isInBlock) {
                console.log(`Transaction included at blockHash ${status.asInBlock}`);
            } else if (status.isFinalized) {
                console.log(`Transaction finalized at blockHash ${status.asFinalized}`);
                unsub.then();
                return resolve();
            } else if (isError) {
                console.log('Transaction error');
                unsub.then();
                return reject();
            }
        });
    });
}

async function setupRelayAsset(api: ApiPromise, signer: KeyringPair, initialBalance = 0n) {
    // The relay asset is registered in the genesis block.

    const assetSetupCalls = [
        api.tx.assetRate.create(RELAY_ASSET_ID, 1_000_000_000_000_000_000n), // 1 on 1
    ];

    if (initialBalance > BigInt(0)) {
        assetSetupCalls.push(
            api.tx.tokens.setBalance(signer.address, RELAY_ASSET_ID, initialBalance, 0)
        );
    }

    const batchCall = api.tx.utility.batch(assetSetupCalls);
    const sudoCall = api.tx.sudo.sudo(batchCall);

    await submitExtrinsic(signer, sudoCall, {});
}

// Transfer the relay chain asset to the parachain specified by paraId.
// Receiver address is same as the sender's.
async function transferRelayAssetToPara(
    amount: bigint,
    paraId: number,
    relayApi: ApiPromise,
    signer: KeyringPair
) {
    const receiverKeypair = new Keyring();
    receiverKeypair.addFromAddress(signer.address);

    // If system parachain we use teleportation, otherwise we do a reserve transfer.
    const transferKind = paraId < 2000 ? 'limitedTeleportAssets' : 'limitedReserveTransferAssets';

    const feeAssetItem = 0;
    const weightLimit = 'Unlimited';
    const reserveTransfer = relayApi.tx.xcmPallet[transferKind](
        {V3: {parents: 0, interior: {X1: {Parachain: paraId}}}}, //dest
        {
            V3: {
                parents: 0,
                interior: {
                    X1: {
                        AccountId32: {
                            chain: 'Any',
                            id: receiverKeypair.pairs[0].publicKey,
                        },
                    },
                },
            },
        }, //beneficiary
        {
            V3: [
                {
                    id: {
                        Concrete: {parents: 0, interior: 'Here'},
                    },
                    fun: {
                        Fungible: amount,
                    },
                },
            ],
        }, //asset
        feeAssetItem,
        weightLimit
    );
    await submitExtrinsic(signer, reserveTransfer, {});
}

async function sleep(milliseconds: number) {
    return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

const getAddressFromModuleId = (moduleId: string): string => {
    if (moduleId.length !== 8) {
        console.log('Module Id must be 8 characters (i.e. `py/trsry`)');
        return '';
    }
    const address = stringToU8a(('modl' + moduleId).padEnd(32, '\0'));
    return encodeAddress(address);
};

const getFreeBalance = async (api: ApiPromise, address: string): Promise<bigint> => {
    const {
        data: {free},
    } = (await api.query.system.account(address)).toJSON() as any;
    return BigInt(free);
};

export {
    RELAY_ASSET_ID,
    setupRelayAsset,
    sleep,
    submitExtrinsic,
    transferRelayAssetToPara,
    getAddressFromModuleId,
    getFreeBalance,
};
