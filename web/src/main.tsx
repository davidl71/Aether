console.log('main.tsx: Script starting...');

import React from 'react';
console.log('main.tsx: React imported:', typeof React);

import ReactDOM from 'react-dom/client';
console.log('main.tsx: ReactDOM imported:', typeof ReactDOM);

import './i18n';
import App from './App';
console.log('main.tsx: App imported:', typeof App);

import './styles/app.css';
console.log('main.tsx: CSS imported');

const rootElement = document.getElementById('root');

if (!rootElement) {
  console.error('Root element not found');
  throw new Error('Root element not found');
}

console.log('React app mounting...');

try {
ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
  console.log('React app mounted successfully');
} catch (error) {
  console.error('Failed to mount React app:', error);
  rootElement.innerHTML = `
    <div style="padding: 40px; color: #ef4444; font-family: system-ui;">
      <h1>Application Error</h1>
      <p>Failed to mount React application.</p>
      <pre style="background: #1a1a1a; padding: 16px; border-radius: 4px; overflow: auto;">${error instanceof Error ? error.stack : String(error)}</pre>
      <p style="margin-top: 16px; font-size: 14px;">Check browser console for details.</p>
    </div>
  `;
}
