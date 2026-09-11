import type { Json, MetadataDocument } from './types';
import { text } from './types.ts';

export const fields = [
  {
    id: 'title',
    tag: 'XMP-dc:Title',
    de: 'Titel',
    en: 'Title',
    type: 'text',
    aliases: ['Title', 'ObjectName'],
  },
  {
    id: 'description',
    tag: 'XMP-dc:Description',
    de: 'Beschreibung',
    en: 'Description',
    type: 'textarea',
    aliases: ['Description', 'ImageDescription', 'Caption-Abstract'],
  },
  {
    id: 'keywords',
    tag: 'XMP-dc:Subject',
    de: 'Keywords',
    en: 'Keywords',
    type: 'list',
    aliases: ['Subject', 'Keywords'],
  },
  {
    id: 'creator',
    tag: 'XMP-dc:Creator',
    de: 'Urheber',
    en: 'Creator',
    type: 'list',
    aliases: ['Creator', 'Artist', 'By-line'],
  },
  {
    id: 'copyright',
    tag: 'XMP-dc:Rights',
    de: 'Copyright',
    en: 'Copyright',
    type: 'text',
    aliases: ['Rights', 'Copyright', 'CopyrightNotice'],
  },
  {
    id: 'rating',
    tag: 'XMP-xmp:Rating',
    de: 'Bewertung',
    en: 'Rating',
    type: 'rating',
    aliases: ['Rating'],
  },
  {
    id: 'email',
    tag: 'XMP-iptcCore:CreatorWorkEmail',
    de: 'E-Mail',
    en: 'Email',
    type: 'text',
    aliases: ['CreatorWorkEmail'],
  },
  {
    id: 'phone',
    tag: 'XMP-iptcCore:CreatorWorkTelephone',
    de: 'Telefon',
    en: 'Phone',
    type: 'text',
    aliases: ['CreatorWorkTelephone'],
  },
  {
    id: 'website',
    tag: 'XMP-iptcCore:CreatorWorkURL',
    de: 'Website',
    en: 'Website',
    type: 'text',
    aliases: ['CreatorWorkURL'],
  },
  {
    id: 'dateTaken',
    tag: 'XMP-exif:DateTimeOriginal',
    de: 'Aufnahmezeit',
    en: 'Capture time',
    type: 'date',
    aliases: ['DateTimeOriginal', 'CreateDate'],
  },
  {
    id: 'latitude',
    tag: 'XMP-exif:GPSLatitude',
    de: 'Breitengrad',
    en: 'Latitude',
    type: 'number',
    aliases: ['GPSLatitude'],
  },
  {
    id: 'longitude',
    tag: 'XMP-exif:GPSLongitude',
    de: 'Längengrad',
    en: 'Longitude',
    type: 'number',
    aliases: ['GPSLongitude'],
  },
  {
    id: 'altitude',
    tag: 'XMP-exif:GPSAltitude',
    de: 'Höhe (m)',
    en: 'Altitude (m)',
    type: 'number',
    aliases: ['GPSAltitude'],
  },
] as const;
export type Field = (typeof fields)[number];
export function fieldValue(
  doc: MetadataDocument | undefined,
  id: string,
): Json {
  if (!doc) return null;
  const field = fields.find((f) => f.id === id);
  if (!field) return null;
  const exact = (source: string) =>
    doc.tags.find(
      (t) => `${t.location}:${t.name}` === field.tag && t.source === source,
    );
  const found =
    exact('sidecar') ??
    exact('embedded') ??
    doc.tags.find(
      (t) =>
        (field.aliases as readonly string[]).includes(t.name) && !t.derived,
    );
  return found?.raw ?? null;
}
export function aggregate(
  docs: MetadataDocument[],
  id: string,
): {
  state: 'same' | 'mixed' | 'missing' | 'partial';
  value: Json;
  missing: number;
} {
  const values = docs.map((d) => fieldValue(d, id));
  const missing = values.filter((v) => v === null || text(v) === '').length;
  if (!values.length || missing === values.length)
    return { state: 'missing', value: null, missing };
  const present = values.filter((v) => v !== null && text(v) !== '');
  const same = present.every(
    (v) => JSON.stringify(v) === JSON.stringify(present[0]),
  );
  return {
    state: missing ? 'partial' : same ? 'same' : 'mixed',
    value: same ? (present[0] ?? null) : null,
    missing,
  };
}
export function renamePreview(
  template: string,
  doc: MetadataDocument,
  sequence: number,
): string {
  const stem = doc.file.name.slice(0, -(doc.file.extension.length + 1));
  const date = text(fieldValue(doc, 'dateTaken'))
    .slice(0, 10)
    .replaceAll(':', '-');
  return template.replace(
    /\{(name|ext|date|seq(?::\d)?)\}/g,
    (_, variable: string) => {
      if (variable === 'name') return stem;
      if (variable === 'ext') return doc.file.extension;
      if (variable === 'date') {
        if (!date) throw new Error(`Missing capture date: ${doc.file.name}`);
        return date;
      }
      return String(sequence).padStart(
        Number(variable.split(':')[1] ?? 3),
        '0',
      );
    },
  );
}
