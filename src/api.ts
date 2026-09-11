import { invoke, isTauri } from '@tauri-apps/api/core';
import type { FileEntry, MetadataDocument } from './domain/types';

export const native = isTauri();
export const review =
  !native &&
  import.meta.env.DEV &&
  new URLSearchParams(location.search).has('review');
export const api = <T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> => {
  if (!native)
    return Promise.reject(
      new Error(
        'Diese Aktion benötigt die Tagryn Desktop-App. / This action requires the Tagryn desktop app.',
      ),
    );
  return invoke<T>(command, args);
};
export async function reviewDocuments(): Promise<{
  files: MetadataDocument[];
  entries?: FileEntry[];
}> {
  if (!review) return { files: [] };
  const response = await fetch(
    new URLSearchParams(location.search).get('review') === 'large'
      ? '/review-large.json'
      : '/review-session.json',
  );
  if (!response.ok)
    throw new Error(
      'Generate a real metadata review session with npm run fixtures.',
    );
  return response.json();
}
