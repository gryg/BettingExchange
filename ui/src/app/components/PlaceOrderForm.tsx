"use client";

import React, { useState } from 'react';
import { useWallet } from '@/app/contexts/WalletContext'; 
import { executePlaceOrder } from '@/lib/contractInteractions'; 
import { ExecutePlaceOrderParams, OrderType, Outcome, Coin as ContractCoinFromTypes } from '@/lib/types';
import { OfflineSigner } from '@cosmjs/proto-signing';

// const BETTING_DENOM = process.env.NEXT_PUBLIC_BETTING_DENOM || "inj";
// It's better to get BETTING_DENOM from contract config or a shared const
const BETTING_DENOM = "inj"; // Or uinj, ensure this matches what your contract expects & SDK sends for funds


interface PlaceOrderFormProps {
    eventId: string; 
    onOrderPlaced: () => void; 
}

const PlaceOrderForm: React.FC<PlaceOrderFormProps> = ({ eventId, onOrderPlaced }) => {
    const { injectiveAddress, signer, isWalletConnected } = useWallet();
    const [orderType, setOrderType] = useState<OrderType>(OrderType.Back);
    const [outcome, setOutcome] = useState<Outcome>(Outcome.Yes);
    const [stake, setStake] = useState(''); 
    const [odds, setOdds] = useState('');   
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const calculateLiability = (): string => {
        if (orderType === OrderType.Lay && parseFloat(odds) > 1 && parseFloat(stake) > 0) {
            const liability = (parseFloat(odds) - 1) * parseFloat(stake);
            return liability.toFixed(6); // Adjust precision
        }
        return stake; 
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);
        if (!isWalletConnected || !signer || !injectiveAddress) {
            setError('Please connect your wallet.');
            return;
        }
        if (!eventId || !stake.trim() || !odds.trim() || parseFloat(odds) <= 1) {
            setError('Please fill in all fields correctly. Odds must be > 1.');
            return;
        }

        setIsLoading(true);

        const params: ExecutePlaceOrderParams = {
            eventId,
            orderType,
            outcome,
            stake, 
            odds,  
        };
        
        let amountToSend = '';
        if (orderType === OrderType.Back) {
            amountToSend = stake;
        } else { 
            const oddsVal = parseFloat(odds);
            const stakeVal = parseFloat(stake);
            if (oddsVal > 1 && stakeVal > 0) {
                amountToSend = ((oddsVal - 1) * stakeVal).toString(); 
            } else {
                setError("Invalid odds or stake for lay bet liability calculation.");
                setIsLoading(false);
                return;
            }
        }
        
        // Ensure amountToSend is in the smallest unit if stake is in main unit
        // e.g., if user enters "1" INJ, amountToSend might need to be "1000000000000000000"
        const funds: ContractCoinFromTypes[] = [{ denom: BETTING_DENOM, amount: amountToSend }];

        try {
            await executePlaceOrder(injectiveAddress, signer as OfflineSigner, params, funds);
            setStake('');
            setOdds('');
            onOrderPlaced(); 
        } catch (err) {
            setError(err instanceof Error ? err.message : "Transaction failed");
        }
        setIsLoading(false);
    };
    
    // JSX remains the same as your summary
    return (
        <form onSubmit={handleSubmit} className="space-y-4 p-6 bg-gray-700 rounded-lg shadow-md mt-6">
            <h3 className="text-lg font-semibold text-white">Place Your Bet for Event ID: {eventId}</h3>
            {error && <p className="text-red-400 bg-red-900 p-2 rounded">{error}</p>}
            <div>
                <label htmlFor="orderType" className="block text-sm font-medium text-gray-300">Bet Type:</label>
                <select
                    id="orderType"
                    value={orderType}
                    onChange={(e) => setOrderType(e.target.value as OrderType)}
                    className="mt-1 block w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm text-white"
                >
                    <option value={OrderType.Back}>Back (Bet For)</option>
                    <option value={OrderType.Lay}>Lay (Bet Against)</option>
                </select>
            </div>
            <div>
                <label htmlFor="outcome" className="block text-sm font-medium text-gray-300">Outcome:</label>
                <select
                    id="outcome"
                    value={outcome}
                    onChange={(e) => setOutcome(e.target.value as Outcome)}
                    className="mt-1 block w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm text-white"
                >
                    <option value={Outcome.Yes}>Yes</option>
                    <option value={Outcome.No}>No</option>
                </select>
            </div>
            <div>
                <label htmlFor="stake" className="block text-sm font-medium text-gray-300">
                    {orderType === OrderType.Back ? "Your Stake:" : "Backer's Stake to Match:"}
                </label>
                <input
                    type="number"
                    id="stake"
                    value={stake}
                    onChange={(e) => setStake(e.target.value)}
                    placeholder="e.g., 100"
                    required
                    min="0.000001"
                    step="any"
                    className="mt-1 block w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm text-white"
                />
            </div>
            <div>
                <label htmlFor="odds" className="block text-sm font-medium text-gray-300">Odds:</label>
                <input
                    type="number"
                    id="odds"
                    value={odds}
                    onChange={(e) => setOdds(e.target.value)}
                    placeholder="e.g., 2.5 (must be > 1.0)"
                    required
                    min="1.000001"
                    step="any"
                    className="mt-1 block w-full px-3 py-2 bg-gray-600 border border-gray-500 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm text-white"
                />
            </div>
            {orderType === OrderType.Lay && parseFloat(odds) > 1 && parseFloat(stake) > 0 && (
                <p className="text-sm text-gray-400">
                    Your potential liability (amount to deposit): {calculateLiability()} {BETTING_DENOM}
                </p>
            )}
             {orderType === OrderType.Back && parseFloat(stake) > 0 && (
                <p className="text-sm text-gray-400">
                    Amount to deposit: {stake} {BETTING_DENOM}
                </p>
            )}
            <button
                type="submit"
                disabled={!isWalletConnected || isLoading}
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
            >
                {isLoading ? 'Placing Order...' : 'Place Order'}
            </button>
        </form>
    );
};

export default PlaceOrderForm;