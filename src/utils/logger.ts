import { invoke } from '@tauri-apps/api/core';

// Store original console methods
const originalConsole = {
  log: console.log,
  info: console.info,
  warn: console.warn,
  error: console.error,
  debug: console.debug
};

// Function to format any type of message
function formatMessage(args: any[]): string {
  return args.map(arg => {
    if (typeof arg === 'object') {
      try {
        return JSON.stringify(arg);
      } catch (e) {
        return String(arg);
      }
    }
    return String(arg);
  }).join(' ');
}

// Override console methods
export function setupConsoleRedirection() {
  console.log = (...args: any[]) => {
    const message = formatMessage(args);
    originalConsole.log(...args); // Still log to browser console
    invoke('log_to_console', { level: 'log', message }).catch(err => {
      originalConsole.error('Failed to send log to Rust backend:', err);
    });
  };

  console.info = (...args: any[]) => {
    const message = formatMessage(args);
    originalConsole.info(...args);
    invoke('log_to_console', { level: 'info', message }).catch(err => {
      originalConsole.error('Failed to send info to Rust backend:', err);
    });
  };

  console.warn = (...args: any[]) => {
    const message = formatMessage(args);
    originalConsole.warn(...args);
    invoke('log_to_console', { level: 'warn', message }).catch(err => {
      originalConsole.error('Failed to send warning to Rust backend:', err);
    });
  };

  console.error = (...args: any[]) => {
    const message = formatMessage(args);
    originalConsole.error(...args);
    invoke('log_to_console', { level: 'error', message }).catch(err => {
      originalConsole.error('Failed to send error to Rust backend:', err);
    });
  };

  console.debug = (...args: any[]) => {
    const message = formatMessage(args);
    originalConsole.debug(...args);
    invoke('log_to_console', { level: 'debug', message }).catch(err => {
      originalConsole.error('Failed to send debug to Rust backend:', err);
    });
  };
}

// Function to restore original console behavior
export function restoreConsole() {
  console.log = originalConsole.log;
  console.info = originalConsole.info;
  console.warn = originalConsole.warn;
  console.error = originalConsole.error;
  console.debug = originalConsole.debug;
} 