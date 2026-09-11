export interface JsonArray extends Array<Json> {}
export interface JsonObject {
  [key: string]: Json;
}
export type Json = string | number | boolean | null | JsonArray | JsonObject;
export interface Fingerprint {
  size: number;
  modifiedNs: string;
  sha256: string;
}
export interface FileEntry {
  id: string;
  path: string;
  name: string;
  folder: string;
  extension: string;
  size: number;
  modified: string;
  readonly: boolean;
}
export interface Tag {
  id: string;
  key: string;
  group: string;
  location: string;
  name: string;
  label: string;
  raw: Json;
  formatted: Json;
  source: string;
  writable: boolean;
  derived: boolean;
  field: string | null;
  format: string | null;
}
export interface MetadataDocument {
  file: FileEntry;
  fingerprint: Fingerprint;
  tags: Tag[];
  capabilities: {
    read: boolean;
    write: string;
    preview: string;
    format: string;
  };
  sidecar: string | null;
  sidecarFingerprint: Fingerprint | null;
  warnings: string[];
}
export interface Edit {
  field: string;
  value: Json;
  mode: 'replace' | 'add' | 'remove' | 'shift';
}
export interface ChangeRequest {
  fileId: string;
  expected: Fingerprint;
  sidecarExpected: Fingerprint | null;
  edits: Edit[];
  privacy: string | null;
  syncLegacy: boolean;
  forceSidecar: boolean;
  rename: string | null;
}
export interface Difference {
  tag: string;
  before: Json;
  after: Json;
  source: string;
}
export interface PlannedFile {
  file: FileEntry;
  target: string;
  differences: Difference[];
  warnings: string[];
  error: string | null;
  renameTo: string | null;
}
export interface Plan {
  id: string;
  files: PlannedFile[];
  created: string;
}
export interface FileResult {
  fileId: string;
  path: string;
  status: string;
  message: string;
  backup: string | null;
  backupFingerprint: Fingerprint | null;
  after: Fingerprint | null;
  originalPath: string | null;
  warnings: string[];
  differences: Difference[];
}
export interface Job {
  id: string;
  kind: string;
  status: string;
  total: number;
  completed: number;
  errors: number;
  results: FileResult[];
  started: string;
}
export interface Preset {
  id: string;
  name: string;
  edits: Edit[];
}
export interface SavedView {
  name: string;
  search: string;
  filter: string;
  columns: string[];
  sort: string;
}
export interface Preferences {
  theme: 'system' | 'light' | 'dark';
  language: 'de' | 'en';
  fontSize: number;
  technical: boolean;
  recursive: boolean;
  sidebar: number;
  inspector: number;
  favorites: string[];
  tagFavorites: string[];
  columns: string[];
  views: SavedView[];
  presets: Preset[];
}
export const text = (value: Json | undefined): string =>
  value === null || value === undefined
    ? ''
    : typeof value === 'string'
      ? value
      : Array.isArray(value)
        ? value.map((v) => text(v)).join(', ')
        : typeof value === 'object'
          ? JSON.stringify(value)
          : String(value);
export const sizeText = (n: number): string =>
  n < 1024
    ? `${n} B`
    : n < 1048576
      ? `${(n / 1024).toFixed(0)} KB`
      : `${(n / 1048576).toFixed(1)} MB`;
export function tagValue(
  doc: MetadataDocument | undefined,
  names: string[],
  raw = false,
): Json {
  if (!doc) return null;
  const tag =
    doc.tags.find((t) => names.includes(t.name) && t.source === 'sidecar') ??
    doc.tags.find((t) => names.includes(t.name) && !t.derived) ??
    doc.tags.find((t) => names.includes(t.name));
  return tag ? (raw ? tag.raw : tag.formatted) : null;
}
