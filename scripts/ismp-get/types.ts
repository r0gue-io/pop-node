export type StateMachine = { Polkadot: number } | { Kusama: number };

export interface Events {
    events: any[];
}

export interface Get {
    source: StateMachine;
    dest: StateMachine;
    nonce: bigint;
    from: string;
    keys: Array<string>;
    height: bigint;
    timeout_timestamp: bigint;
}

export type IsmpRequest = { get: Get };

export const API_TYPES = {
    HashAlgorithm: {
        _enum: ["Keccak", "Blake2"],
    },
    StateMachineProof: {
        hasher: "HashAlgorithm",
        storage_proof: "Vec<Vec<u8>>",
    },
    SubstrateStateProof: {
        _enum: {
            OverlayProof: "StateMachineProof",
            StateProof: "StateMachineProof",
        },
    },
    LeafIndexQuery: {
        commitment: "H256",
    },
    StateMachine: {
        _enum: {
            Ethereum: "Vec<u8>",
            Polkadot: "Vec<u8>",
            Kusama: "u32",
        },
    },
    Get: {
        // Can't be decoded directly to StateMachine type.
        source: "String",
        // Can't be decoded directly to StateMachine type.
        dest: "String",
        nonce: "u64",
        from: "Vec<u8>",
        keys: "Vec<Vec<u8>>",
        height: "u64",
        timeout_timestamp: "u64",
    },
    Request: {
        _enum: {
            Post: "Post",
            Get: "Get",
        },
    },
};

export const CUSTOM_RPC = {
    ismp: {
        queryRequests: {
            description: "",
            params: [
                {
                    name: "query",
                    type: "Vec<LeafIndexQuery>",
                },
            ],
            type: "Vec<Request>",
        },
    },
};