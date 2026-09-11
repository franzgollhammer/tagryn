import test from 'node:test';
import assert from 'node:assert/strict';
import { importRows, mapImport } from '../src/domain/import.ts';
const files = [
  { id: 'a', name: 'Änderung.jpg', path: '/photos/Änderung.jpg' },
  { id: 'b', name: 'same.jpg', path: '/one/same.jpg' },
  { id: 'c', name: 'same.jpg', path: '/two/same.jpg' },
];
test('import maps only selected fields and keeps Unicode', () => {
  const rows = importRows([
    {
      filename: 'Änderung.jpg',
      subject: 'Eins, Zwei',
      rating: '5',
      ignored: '<script>',
    },
  ]);
  const result = mapImport(
    rows,
    'filename',
    { subject: 'keywords', rating: 'rating' },
    files,
  );
  assert.deepEqual(result, [
    {
      fileId: 'a',
      edits: [
        { field: 'keywords', value: ['Eins', 'Zwei'], mode: 'replace' },
        { field: 'rating', value: 5, mode: 'replace' },
      ],
    },
  ]);
});
test('ambiguous filenames and duplicate rows fail before staging', () => {
  assert.throws(
    () =>
      mapImport(
        [{ file: 'same.jpg', title: 'x' }],
        'file',
        { title: 'title' },
        files,
      ),
    /ambiguous/,
  );
  assert.throws(
    () =>
      mapImport(
        [
          { file: 'Änderung.jpg', title: 'a' },
          { file: 'Änderung.jpg', title: 'b' },
        ],
        'file',
        { title: 'title' },
        files,
      ),
    /Duplicate import row/,
  );
});
test('invalid numbers, mappings and nested write values are rejected', () => {
  assert.throws(
    () =>
      mapImport(
        [{ file: 'Änderung.jpg', rating: 'invalid' }],
        'file',
        { rating: 'rating' },
        files,
      ),
    /numeric/,
  );
  assert.throws(
    () =>
      mapImport(
        [{ file: 'Änderung.jpg', a: 'a', b: 'b' }],
        'file',
        { a: 'title', b: 'title' },
        files,
      ),
    /only once/,
  );
  assert.throws(
    () =>
      mapImport(
        [{ file: 'Änderung.jpg', title: { command: 'danger' } }],
        'file',
        { title: 'title' },
        files,
      ),
    /nested objects/,
  );
});
