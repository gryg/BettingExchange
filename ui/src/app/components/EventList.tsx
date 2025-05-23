"use client";

import React from 'react';
import { Event as ContractEvent } from '@/lib/types'; // Using @ alias assuming your tsconfig paths are set

// Define the props for EventList, including onSelectEvent
interface EventListProps {
    events: ContractEvent[];
    onSelectEvent: (eventId: string) => void; // Add this line
}

const EventList: React.FC<EventListProps> = ({ events, onSelectEvent }) => {
    if (!events || events.length === 0) {
        return <p className="text-gray-400">No events found.</p>;
    }

    return (
        <ul className="space-y-4">
            {events.map((event) => (
                <li key={event.id} className="p-4 bg-gray-800 rounded-lg shadow">
                    <h3 className="text-xl font-semibold text-indigo-400">Event ID: {event.id}</h3>
                    <p className="text-gray-300">{event.description}</p>
                    <p className="text-sm text-gray-500">Oracle: {event.oracle}</p>
                    <p className="text-sm text-gray-500">Status: {event.status}</p>
                    <button 
                        onClick={() => onSelectEvent(event.id)} // Call the handler
                        className="mt-2 px-3 py-1 text-xs font-medium text-white bg-blue-600 rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 focus:ring-offset-gray-900"
                    >
                        Place Bet on this Event
                    </button>
                </li>
            ))}
        </ul>
    );
};

export default EventList;