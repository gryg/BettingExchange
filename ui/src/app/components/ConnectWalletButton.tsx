"use client"; // For Next.js App Router client components

import React from 'react';
import { useWallet } from '../contexts/WalletContext'; // Adjust path if your contexts folder is elsewhere

const ConnectWalletButton: React.FC = () => {
    const { injectiveAddress, connectWallet, disconnectWallet, isWalletConnected } = useWallet();

    if (isWalletConnected) {
        return (
            <div className="flex flex-col items-center space-y-2 md:flex-row md:space-y-0 md:space-x-4">
                <p className="text-sm text-green-400">
                    Connected: 
                    <span className="font-mono ml-2 bg-gray-700 px-2 py-1 rounded">
                        {injectiveAddress.substring(0, 9)}...{injectiveAddress.substring(injectiveAddress.length - 4)}
                    </span>
                </p>
                <button 
                    onClick={disconnectWallet}
                    className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 focus:ring-offset-gray-900"
                >
                    Disconnect Wallet
                </button>
            </div>
        );
    }

    return (
        <button 
            onClick={connectWallet}
            className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 focus:ring-offset-gray-900"
        >
            Connect Keplr Wallet
        </button>
    );
};

export default ConnectWalletButton;