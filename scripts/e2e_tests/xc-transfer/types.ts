export type StateMachine = { Polkadot: number } | { Kusama: number };

export interface Get {
    source: StateMachine;
    dest: StateMachine;
    nonce: bigint;
    from: string;
    keys: Array<string>;
    height: bigint;
    timeout_timestamp: bigint;
}

export type IsmpRequest = { post: any } | { get: Get };

export const REGIONX_API_TYPES = {
    CoreIndex: "u32",
    CoreMask: "Vec<u8>",
    Timeslice: "u32",
    RegionId: {
        begin: "Timeslice",
        core: "CoreIndex",
        mask: "CoreMask",
    },
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
    Post: {},
    Get: {
        source: "String",
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

export const REGIONX_CUSTOM_RPC = {
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
