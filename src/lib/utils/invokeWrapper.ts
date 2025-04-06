import { invoke } from '@tauri-apps/api/core';
import { showErrorToast } from '$lib/stores/notifications';

// Interface matching the Rust CommandError structure (simplified)
// We mainly care about the message for display purposes.
interface CommandError {
  code?: string; // e.g., Database, Storage, Validation
  message: string;
}

/**
 * Wraps the Tauri invoke call to provide consistent error handling.
 * On success, returns the result of the command.
 * On failure, logs the error, shows an error toast to the user, and returns null.
 *
 * @param cmd The Tauri command to invoke.
 * @param args Optional arguments for the command.
 * @returns The result of the command on success, or null on failure.
 */
export async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    // Tauri's invoke automatically handles the Result<T, E> from Rust.
    // If Rust returns Err(E), invoke will reject the promise with E.
    // If Rust returns Ok(T), invoke will resolve the promise with T.
    const result = await invoke<T>(cmd, args);
    console.log(`Command '${cmd}' succeeded.`); // Optional: Log success
    return result;

  } catch (error) {
    console.error(`Error invoking command '${cmd}':`, error);

    let errorMessage = 'An unexpected error occurred.';

    // Attempt to parse the error as our structured CommandError from Rust
    if (typeof error === 'object' && error !== null && 'message' in error && typeof error.message === 'string') {
      const cmdError = error as CommandError;
      errorMessage = cmdError.message; // Use the message from the structured error
    } else if (typeof error === 'string') {
      // Handle plain string errors (less ideal, but fallback)
      errorMessage = error;
    }
    // else: Keep the default "An unexpected error occurred."

    // Show error toast to the user
    showErrorToast(errorMessage);

    return null; // Indicate failure to the caller
  }
}