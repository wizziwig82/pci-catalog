/// <reference types="@tauri-apps/api" />

declare module '@tauri-apps/api/tauri' {
  function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

declare module '@tauri-apps/api/dialog' {
  function open(options?: OpenDialogOptions): Promise<string | string[] | null>;
  function save(options?: SaveDialogOptions): Promise<string | null>;
  function message(message: string, options?: MessageDialogOptions): Promise<void>;
  function confirm(message: string, options?: ConfirmDialogOptions): Promise<boolean>;
  
  interface DialogFilter {
    name: string;
    extensions: string[];
  }
  
  interface OpenDialogOptions {
    multiple?: boolean;
    directory?: boolean;
    recursive?: boolean;
    defaultPath?: string;
    filters?: DialogFilter[];
    title?: string;
  }
  
  interface SaveDialogOptions {
    defaultPath?: string;
    filters?: DialogFilter[];
    title?: string;
  }
  
  interface MessageDialogOptions {
    title?: string;
    type?: 'info' | 'warning' | 'error';
  }
  
  interface ConfirmDialogOptions {
    title?: string;
    type?: 'info' | 'warning' | 'error';
  }
} 