"use client";

import React, { useEffect, useState } from 'react';
import ConnectWalletButton from './components/ConnectWalletButton';
import CreateEventForm from './components/CreateEventForm';
import EventList from './components/EventList';
import PlaceOrderForm from './components/PlaceOrderForm'; // Import PlaceOrderForm
import { useWallet } from './contexts/WalletContext';
import { queryListEvents } from '../lib/contractInteractions'; // Use @/lib/... if alias is set up
import { Event as ContractEvent } from '../lib/types';    // Use @/lib/...

export default function HomePage() {
    const { injectiveAddress, signer, isWalletConnected } = useWallet();
    const [events, setEvents] = useState<ContractEvent[]>([]);
    const [isLoadingEvents, setIsLoadingEvents] = useState(false);
    const [selectedEventIdForOrder, setSelectedEventIdForOrder] = useState<string | null>(null);


    const fetchEvents = async () => {
        setIsLoadingEvents(true);
        try {
            const result = await queryListEvents();
            if (result && result.events) {
                setEvents(result.events);
            } else {
                setEvents([]);
            }
        } catch (error) {
            console.error("Failed to fetch events:", error);
            setEvents([]);
        }
        setIsLoadingEvents(false);
    };

    useEffect(() => {
        fetchEvents();
    }, []);

    const handleEventCreated = () => {
        fetchEvents(); // Refresh event list
    }

    const handleOrderPlaced = () => {
        fetchEvents(); // Or more specific refresh logic
        setSelectedEventIdForOrder(null); // Close/reset form
    }


    return (
        <main className="flex min-h-screen flex-col items-center justify-start p-8 sm:p-12 md:p-24 bg-gray-900 text-white">
            <div className="z-10 w-full max-w-5xl items-center justify-between font-mono text-sm lg:flex mb-8">
                <h1 className="text-4xl font-bold mb-4 text-center lg:text-left">Injective Betting Exchange</h1>
                <ConnectWalletButton />
            </div>

            {isWalletConnected && (
                <div className="mb-8 w-full max-w-2xl p-4 bg-gray-800 rounded-lg">
                    <p className="text-sm">Connected: <span className="font-semibold">{injectiveAddress}</span></p>
                </div>
            )}

            <div className="w-full max-w-2xl mb-12">
                {/* CreateEventForm no longer needs injectiveAddress and signer as props */}
                <CreateEventForm onEventCreated={handleEventCreated} />
            </div>
            
            <div className="w-full max-w-2xl">
                <h2 className="text-2xl font-semibold mb-4">Current Events</h2>
                {isLoadingEvents ? (
                    <p>Loading events...</p>
                 ) : (
                    <EventList events={events} onSelectEvent={setSelectedEventIdForOrder} /> // Pass a handler to select event
                 )}
                {selectedEventIdForOrder && isWalletConnected && (
                    <PlaceOrderForm
                        eventId={selectedEventIdForOrder}
                        onOrderPlaced={handleOrderPlaced}
                    />
                )}
            </div>
        </main>
    );
}