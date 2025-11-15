import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles/app.css';

// Register service worker for PWA
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    // VitePWA plugin handles registration automatically
    // This is just for manual update checks if needed
    navigator.serviceWorker.ready.then(() => {
      console.log('Service Worker ready');
    });
  });
}

const rootElement = document.getElementById('root');

if (!rootElement) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
