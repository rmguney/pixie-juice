import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

// Offline use: the service worker stores the app's files (see serviceWorker.ts).
// Registered once the page has loaded, so storing them doesn't slow the first
// visit; if registration fails, the app still works online.
if (import.meta.env.PROD && 'serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {});
  });
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
