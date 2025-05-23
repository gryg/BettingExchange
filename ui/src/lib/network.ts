import {
    ChainGrpcWasmApi,
    ChainRestAuthApi,
    ChainRestTendermintApi,
} from '@injectivelabs/sdk-ts';
import { getNetworkEndpoints, Network } from '@injectivelabs/networks'; // Use Network enum
import { ChainId } from '@injectivelabs/ts-types';

const networkName = process.env.NEXT_PUBLIC_NETWORK_NAME || 'testnet';
// Use Network enum values directly
export const injectiveNetwork = networkName === 'mainnet' ? Network.MainnetK8s : Network.TestnetK8s;

export const endpoints = getNetworkEndpoints(injectiveNetwork);

export const chainGrpcWasmApi = new ChainGrpcWasmApi(endpoints.grpc);
export const chainRestAuthApi = new ChainRestAuthApi(endpoints.rest);
export const chainRestTendermintApi = new ChainRestTendermintApi(endpoints.rest);

export const contractAddress = process.env.NEXT_PUBLIC_CONTRACT_ADDRESS!;
export const chainId = (process.env.NEXT_PUBLIC_INJECTIVE_CHAIN_ID as ChainId)!;

if (!contractAddress) {
    console.warn("NEXT_PUBLIC_CONTRACT_ADDRESS is not set in .env.local. Contract interactions will fail.");
}
if (!chainId) {
    console.warn("NEXT_PUBLIC_INJECTIVE_CHAIN_ID is not set in .env.local. Wallet interactions might fail.");
}