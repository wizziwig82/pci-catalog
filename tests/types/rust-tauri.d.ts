/**
 * Type declarations for Rust Tauri backend functions 
 */

declare module '../../../../src-tauri/src/storage/mongodb' {
  export type Writer = {
    name: string;
    percentage: number;
  };
  
  export type Publisher = {
    name: string;
    percentage: number;
  };
  
  export type TrackPaths = {
    original: string;
    medium: string;
    low: string;
  };
  
  export type Track = {
    title: string;
    album_id: string;
    filename: string;
    duration: number;
    comments?: string;
    writers: Writer[];
    publishers: Publisher[];
    genre?: string;
    instruments: string[];
    mood?: string;
    path: TrackPaths;
  };
  
  export type Album = {
    name: string;
    art_path?: string;
    track_ids: string[];
  };

  export type MongoCredentials = {
    uri: string;
  };

  export type MongoClient = any;

  export type DbResponse<T> = {
    success: boolean;
    message?: string;
    id?: string;
    data?: T;
  };

  export function initialize_mongo_client(credentials: MongoCredentials): Promise<MongoClient>;
  export function create_album(client: MongoClient, album_id: string, album_data: Album): Promise<DbResponse<void>>;
  export function get_album(client: MongoClient, album_id: string): Promise<DbResponse<Album>>;
  export function update_album(client: MongoClient, album_id: string, album_data: Album): Promise<DbResponse<void>>;
  export function delete_album(client: MongoClient, album_id: string): Promise<DbResponse<void>>;
  export function create_track(client: MongoClient, track_id: string, track_data: Track): Promise<DbResponse<void>>;
  export function get_track(client: MongoClient, track_id: string): Promise<DbResponse<Track>>;
  export function update_track(client: MongoClient, track_id: string, track_data: Track): Promise<DbResponse<void>>;
  export function delete_track(client: MongoClient, track_id: string): Promise<DbResponse<void>>;
  export function search_tracks(client: MongoClient, query: string): Promise<DbResponse<Track[]>>;
  export function search_albums(client: MongoClient, query: string): Promise<DbResponse<Album[]>>;
  export function get_tracks_by_album(client: MongoClient, album_id: string): Promise<DbResponse<Track[]>>;
  export function get_all_albums(client: MongoClient): Promise<DbResponse<Album[]>>;
} 