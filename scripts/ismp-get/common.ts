import {ApiPromise, Keyring} from '@polkadot/api';
import {SignerOptions, SubmittableExtrinsic} from '@polkadot/api/types';
import {KeyringPair} from '@polkadot/keyring/types';
import {stringToU8a} from '@polkadot/util';
import {encodeAddress} from '@polkadot/util-crypto';
import {Events} from './types';

const RELAY_ASSET_ID = 1;

async function submitExtrinsic(
    signer: KeyringPair,
    call: SubmittableExtrinsic<'promise'>,
    options: Partial<SignerOptions>,
    tx_events: Events
): Promise<void> {
    return new Promise((resolve, reject) => {
        const unsub = call.signAndSend(signer, options, ({status, isError, events = []}) => {
            console.log(`Current status is ${status}`);
            if (status.isInBlock) {
                console.log(`Transaction included at blockHash ${status.asInBlock}`);
                tx_events.events = events;
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
    await submitExtrinsic(signer, reserveTransfer, {}, {events: []});
}

export {
    RELAY_ASSET_ID,
    submitExtrinsic,
    transferRelayAssetToPara,
};