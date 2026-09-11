import { fields, fieldValue } from './fields.ts';
import type { FileEntry, Edit, Json, MetadataDocument } from './types';
export type ImportRow = Record<string, Json>;
export function importRows(input: unknown): ImportRow[] {
  if (
    typeof input === 'object' &&
    input &&
    'schema' in input &&
    input.schema === 'tagryn.metadata/v1' &&
    'files' in input &&
    Array.isArray(input.files)
  ) {
    return (input.files as MetadataDocument[]).map((doc) =>
      Object.fromEntries([
        ['file', doc.file.path],
        ...fields.map((field) => [field.id, fieldValue(doc, field.id)]),
      ]),
    );
  }
  if (
    !Array.isArray(input) ||
    input.some(
      (row) => typeof row !== 'object' || row === null || Array.isArray(row),
    )
  )
    throw new Error(
      'Import must be a JSON array of rows, a Tagryn JSON export, or CSV with a header.',
    );
  if (input.length > 10000)
    throw new Error('Import contains more than 10,000 rows.');
  return input as ImportRow[];
}
export function mapImport(
  rows: ImportRow[],
  fileColumn: string,
  mapping: Record<string, string>,
  files: FileEntry[],
): { fileId: string; edits: Edit[] }[] {
  if (!fileColumn || !Object.values(mapping).some(Boolean))
    throw new Error('Map a filename column and at least one metadata field.');
  const seen = new Set<string>();
  const fieldIds = Object.values(mapping).filter(Boolean);
  if (new Set(fieldIds).size !== fieldIds.length)
    throw new Error('Each metadata field can be mapped only once.');
  return rows.map((row, index) => {
    const path = String(row[fileColumn] ?? '');
    const matching = files.filter(
      (file) => file.path === path || file.name === path,
    );
    if (matching.length !== 1)
      throw new Error(
        `Row ${index + 1}: filename is missing, unopened or ambiguous: ${path}`,
      );
    const file = matching[0]!;
    if (seen.has(file.id)) throw new Error(`Duplicate import row for ${path}`);
    seen.add(file.id);
    const edits: Edit[] = [];
    for (const [column, id] of Object.entries(mapping)) {
      if (!id || id === fileColumn) continue;
      const field = fields.find((field) => field.id === id);
      if (!field) throw new Error(`Unsupported field ${id}`);
      let value = row[column] ?? null;
      if (field.type === 'list' && typeof value === 'string')
        value = value
          .split(/[,\n]/)
          .map((s) => s.trim())
          .filter(Boolean);
      if (
        ['number', 'rating'].includes(field.type) &&
        value !== null &&
        value !== ''
      ) {
        const n = Number(value);
        if (!Number.isFinite(n))
          throw new Error(`Row ${index + 1}: ${column} must be numeric`);
        value = n;
      }
      if (typeof value === 'object' && value !== null && !Array.isArray(value))
        throw new Error(`Row ${index + 1}: nested objects cannot be written`);
      edits.push({ field: id, value, mode: 'replace' });
    }
    return { fileId: file.id, edits };
  });
}
