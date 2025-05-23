import {
    MsgExecuteContractCompat,
} from '@injectivelabs/sdk-ts';
import {
    MsgBroadcaster,
    Wallet, // Wallet enum for WalletStrategy
    WalletStrategy, // For configuring MsgBroadcaster
} from '@injectivelabs/wallet-ts';
import { chainGrpcWasmApi, contractAddress, chainId, injectiveNetwork } from './network';
import { EventsResponse, ExecuteCreateEventParams, ExecutePlaceOrderParams, Coin as ContractCoinFromTypes } from './types'; // Corrected import
import { OfflineSigner } from '@cosmjs/proto-signing'; // Unified OfflineSigner
import { BigNumberInBase } from '@injectivelabs/utils'; // This should be fine

// --- QUERY FUNCTIONS ---
export const queryListEvents = async (
    startAfter?: string,
    limit?: number
): Promise<EventsResponse | null> => {
    try {
        const query = { list_events: { start_after: startAfter, limit } };
        const queryJson = JSON.stringify(query);
        const base64Query = Buffer.from(queryJson).toString('base64'); // Use Buffer

        const response = await chainGrpcWasmApi.fetchSmartContractState(
            contractAddress,
            base64Query
        );
        if (response.data) {
            const decodedJson = Buffer.from(response.data).toString('utf-8'); // Use Buffer
            const decoded = JSON.parse(decodedJson);
            return decoded as EventsResponse;
        }
    } catch (e) {
        console.error("Error querying list_events:", e);
    }
    return null;
};

// --- EXECUTE FUNCTIONS ---
export const executeCreateEvent = async (
    injectiveAddress: string, 
    signer: OfflineSigner, 
    params: ExecuteCreateEventParams // Corrected type name
) => {
    if (!contractAddress) throw new Error("Contract address not configured");

    const msg = MsgExecuteContractCompat.fromJSON({
        contractAddress: contractAddress,
        sender: injectiveAddress,
        msg: {
            create_event: {
                description: params.description,
                oracle_addr: params.oracleAddr,
                resolution_deadline: params.resolutionDeadline,
            },
        },
    });

    const walletStrategy = new WalletStrategy({ // Corrected initialization
        chainId: chainId,
        wallet: Wallet.Keplr, 
    });

    const msgBroadcaster = new MsgBroadcaster({
        walletStrategy,
        network: injectiveNetwork,
    });

    try {
        const txResponse = await msgBroadcaster.broadcast({
            address: injectiveAddress, 
            msgs: msg,
            // Signer is usually handled by WalletStrategy when using Keplr
        });
        console.log('Create Event Tx Response:', txResponse);
        if (txResponse.code !== 0) {
            throw new Error(`Transaction failed: ${txResponse.rawLog || txResponse.txHash}`);
        }
        alert(`Event created! TxHash: ${txResponse.txHash}`);
        return txResponse;
    } catch (e) {
        console.error("Error executing create_event:", e);
        alert(`Error: ${e instanceof Error ? e.message : String(e)}`);
        throw e;
    }
};

export const executePlaceOrder = async (
    injectiveAddress: string,
    signer: OfflineSigner, 
    params: ExecutePlaceOrderParams, // Ensure this is imported correctly from ./types
    funds?: ContractCoinFromTypes[]  // Ensure this is imported correctly from ./types
) => {
    if (!contractAddress) throw new Error("Contract address not configured");

    const msg = MsgExecuteContractCompat.fromJSON({
        contractAddress: contractAddress,
        sender: injectiveAddress,
        msg: {
            place_order: {
                event_id: params.eventId,
                order_type: params.orderType,
                outcome: params.outcome,
                stake: params.stake, 
                odds: params.odds,   
            },
        },
        funds: funds?.map(coin => ({ 
            denom: coin.denom,
            amount: new BigNumberInBase(coin.amount).toWei().toFixed(), 
        })),
    });

    const walletStrategy = new WalletStrategy({ // Corrected initialization
        chainId: chainId,
        wallet: Wallet.Keplr,
    });
    
    const msgBroadcaster = new MsgBroadcaster({
        walletStrategy,
        network: injectiveNetwork,
    });

    try {
        const txResponse = await msgBroadcaster.broadcast({
            address: injectiveAddress,
            msgs: msg,
        });
        console.log('Place Order Tx Response:', txResponse);
        if (txResponse.code !== 0) { 
            throw new Error(`Transaction failed: ${txResponse.rawLog || txResponse.txHash}`);
        }
        alert(`Order placement transaction sent! TxHash: ${txResponse.txHash}`);
        return txResponse;
    } catch (e) {
        console.error("Error executing place_order:", e);
        alert(`Order Placement Error: ${e instanceof Error ? e.message : String(e)}`);
        throw e;
    }
};