"use client";
import React, { useState } from 'react';
import { executeCreateEvent } from '@/lib/contractInteractions'; // Use alias
import { ExecuteCreateEventParams } from '@/lib/types';      // Use alias
import { OfflineSigner } from '@cosmjs/proto-signing';      // Use unified signer type
import { useWallet } from '@/app/contexts/WalletContext';     // Use alias

interface CreateEventFormProps {
    onEventCreated: () => void; 
}

const CreateEventForm: React.FC<CreateEventFormProps> = ({ onEventCreated }) => {
    const { injectiveAddress, signer, isWalletConnected } = useWallet(); // Get from context
    const [description, setDescription] = useState('');
    const [oracleAddr, setOracleAddr] = useState(''); 
    const [deadline, setDeadline] = useState(''); 
    const [isLoading, setIsLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!isWalletConnected || !signer || !injectiveAddress) {
            alert('Please connect your wallet first.');
            return;
        }
        if (!description.trim()) {
            alert('Description cannot be empty.');
            return;
        }

        setIsLoading(true);
        try {
            const params: ExecuteCreateEventParams = { description };
            if (oracleAddr.trim()) params.oracleAddr = oracleAddr.trim();
            if (deadline.trim()) {
                const deadlineDate = new Date(deadline);
                params.resolutionDeadline = Math.floor(deadlineDate.getTime() / 1000).toString();
            }
            
            await executeCreateEvent(injectiveAddress, signer as OfflineSigner, params);
            alert('Event creation transaction sent!');
            setDescription('');
            setOracleAddr('');
            setDeadline('');
            onEventCreated(); 
        } catch (error) {
            console.error('Failed to create event:', error);
        }
        setIsLoading(false);
    };

    // JSX remains the same as your summary
    return (
        <form onSubmit={handleSubmit} className="space-y-4 p-6 bg-gray-800 rounded-lg shadow-md">
            <h2 className="text-xl font-semibold text-white mb-4">Create New Event</h2>
            <div>
                <label htmlFor="description" className="block text-sm font-medium text-gray-300">Description:</label>
                <input
                    type="text"
                    id="description"
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    required
                    className="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm text-white"
                />
            </div>
            <div>
                <label htmlFor="oracleAddr" className="block text-sm font-medium text-gray-300">Oracle Address (Optional):</label>
                <input
                    type="text"
                    id="oracleAddr"
                    value={oracleAddr}
                    onChange={(e) => setOracleAddr(e.target.value)}
                    className="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm text-white"
                    placeholder="inj..."
                />
            </div>
            <div>
                <label htmlFor="deadline" className="block text-sm font-medium text-gray-300">Resolution Deadline (Optional):</label>
                <input
                    type="datetime-local"
                    id="deadline"
                    value={deadline}
                    onChange={(e) => setDeadline(e.target.value)}
                    className="mt-1 block w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm text-white"
                />
            </div>
            <button
                type="submit"
                disabled={!isWalletConnected || isLoading}
                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
            >
                {isLoading ? 'Creating...' : 'Create Event'}
            </button>
        </form>
    );
};

export default CreateEventForm;