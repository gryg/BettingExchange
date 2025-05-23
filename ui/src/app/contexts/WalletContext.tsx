"use client";

import React, { createContext, useState, useContext, ReactNode, useCallback } from 'react';
import { ChainId, AccountAddress } from '@injectivelabs/ts-types';
import { Keplr } from '@keplr-wallet/types'; // For Keplr type
import { OfflineSigner } from '@cosmjs/proto-signing'; // Unified OfflineSigner
import { Network, getNetworkEndpoints } from '@injectivelabs/networks';

interface WalletContextState {
    address: AccountAddress | undefined;
    injectiveAddress: string;
    signer: OfflineSigner | undefined;
    connectWallet: () => Promise<void>;
    disconnectWallet: () => void;
    isWalletConnected: boolean;
    chainId: ChainId;
}

const WalletContext = createContext<WalletContextState | undefined>(undefined);

export const WalletProvider = ({ children }: { children: ReactNode }) => {
    const [address, setAddress] = useState<AccountAddress | undefined>(undefined);
    const [injectiveAddress, setInjectiveAddress] = useState<string>('');
    const [signer, setSigner] = useState<OfflineSigner | undefined>(undefined);
    const [isWalletConnected, setIsWalletConnected] = useState<boolean>(false);

    const chainId = (process.env.NEXT_PUBLIC_INJECTIVE_CHAIN_ID as ChainId) || 'injective-888'; // Default to testnet chainId string

    const connectWallet = useCallback(async () => {
        try {
            const keplr = (window as any).keplr as Keplr | undefined;
            if (!keplr) {
                alert("Please install Keplr extension.");
                return;
            }

            await keplr.enable(chainId);
            const offlineSigner = keplr.getOfflineSigner(chainId); // This is an OfflineSigner
            const accounts = await offlineSigner.getAccounts();
            
            if (accounts.length > 0) {
                const accountAddress = accounts[0].address as AccountAddress;
                setAddress(accountAddress); 
                setInjectiveAddress(accountAddress);

                setSigner(offlineSigner);
                setIsWalletConnected(true);
            } else {
                alert("No accounts found in Keplr or permission denied.");
            }
        } catch (error) {
            console.error("Error connecting to Keplr:", error);
            alert(`Error connecting wallet: ${error instanceof Error ? error.message : String(error)}`);
        }
    }, [chainId]);

    const disconnectWallet = useCallback(() => {
        setAddress(undefined);
        setInjectiveAddress('');
        setSigner(undefined);
        setIsWalletConnected(false);
    }, []);

    return (
        <WalletContext.Provider value={{ address, injectiveAddress, signer, connectWallet, disconnectWallet, isWalletConnected, chainId }}>
            {children}
        </WalletContext.Provider>
    );
};

export const useWallet = () => {
    const context = useContext(WalletContext);
    if (!context) {
        throw new Error('useWallet must be used within a WalletProvider');
    }
    return context;
};