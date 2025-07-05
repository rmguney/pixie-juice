'use client';

import { useState } from 'react';

export default function TestPage() {
  const [message, setMessage] = useState('Hello World');
  
  return (
    <div className="min-h-screen bg-black text-white flex items-center justify-center">
      <div>
        <h1>Test Page</h1>
        <p>{message}</p>
        <button 
          onClick={() => setMessage('Button clicked!')}
          className="px-4 py-2 bg-white text-black rounded"
        >
          Test Button
        </button>
      </div>
    </div>
  );
}
