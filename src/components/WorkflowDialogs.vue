<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { open, confirm } from '@tauri-apps/plugin-dialog';
import {
  AlertTriangle,
  ArrowRight,
  Check,
  Copy,
  Download,
  FileJson,
  FolderOpen,
  History,
  Layers,
  LockKeyhole,
  MapPin,
  Pencil,
  Plus,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  Trash2,
  Keyboard,
  Sun,
  Moon,
  Monitor,
} from '@lucide/vue';
import Modal from './Modal.vue';
import Comparison from './Comparison.vue';
import { useWorkspace } from '../stores/workspace';
import { useI18n } from '../i18n';
import { aggregate, fieldValue, fields, renamePreview } from '../domain/fields';
import type { Edit, Json, MetadataDocument, Preset } from '../domain/types';
import { text } from '../domain/types';
import { importRows, mapImport } from '../domain/import';
import type { ImportRow } from '../domain/import';
import { api, native, review } from '../api';

const store = useWorkspace();
const { t, de } = useI18n();
const enabled = ref<string[]>([]);
const values = ref<Record<string, string>>({});
const modes = ref<Record<string, Edit['mode']>>({});
const sync = ref(false);
const sidecar = ref(false);
const privacy = ref('location');
const reviewIndex = ref(0);
const commandQuery = ref('');
const renameRule = ref('{date}_{seq:3}.{ext}');
const shiftSeconds = ref(0);
const transferFields = ref(['creator', 'copyright']);
const sourceId = ref('');
const newName = ref('');
const rows = ref<ImportRow[]>([]);
const mapping = ref<Record<string, string>>({});
const fileColumn = ref('file');
const importName = ref('');
const error = ref('');
const currentPlan = computed(() => store.plan?.files[reviewIndex.value]);
const planErrors = computed(
  () => store.plan?.files.filter((file) => file.error).length ?? 0,
);
const planCount = computed(
  () =>
    store.plan?.files.reduce((sum, file) => sum + file.differences.length, 0) ??
    0,
);
const canEdit = computed(() =>
  store.selectedDocuments.some(
    (doc) => !doc.file.readonly && doc.capabilities.write !== 'readOnly',
  ),
);
const actions = computed(() => [
  {
    id: 'open',
    label: t('Dateien öffnen', 'Open files'),
    icon: Plus,
    key: '⌘ O',
    run: () => store.openFiles(),
    disabled: false,
  },
  {
    id: 'folder',
    label: t('Ordner öffnen', 'Open folder'),
    icon: FolderOpen,
    key: '⇧ ⌘ O',
    run: () => store.openFiles(true),
    disabled: false,
  },
  {
    id: 'edit',
    label: t('Metadaten bearbeiten', 'Edit metadata'),
    icon: Pencil,
    key: '⌘ E',
    run: () => show('edit'),
    disabled: !canEdit.value,
  },
  {
    id: 'compare',
    label: t('Dateien vergleichen', 'Compare files'),
    icon: Layers,
    key: '⌘ D',
    run: () => show('compare'),
    disabled: store.selected.length < 2,
  },
  {
    id: 'privacy',
    label: t('Datenschutz prüfen', 'Review privacy'),
    icon: ShieldCheck,
    key: '',
    run: () => show('privacy'),
    disabled: !store.selected.length,
  },
  {
    id: 'transfer',
    label: t('Metadaten übertragen', 'Transfer metadata'),
    icon: Copy,
    key: '',
    run: () => show('transfer'),
    disabled: store.selected.length < 2,
  },
  {
    id: 'time',
    label: t('Aufnahmezeiten verschieben', 'Shift capture times'),
    icon: History,
    key: '',
    run: () => show('time'),
    disabled: !canEdit.value,
  },
  {
    id: 'rename',
    label: t('Stapelumbenennung', 'Batch rename'),
    icon: Pencil,
    key: '',
    run: () => show('rename'),
    disabled: !store.selected.length,
  },
  {
    id: 'presets',
    label: t('Vorlagen', 'Presets'),
    icon: SlidersHorizontal,
    key: '',
    run: () => show('presets'),
    disabled: false,
  },
  {
    id: 'export',
    label: t('Metadaten exportieren', 'Export metadata'),
    icon: Download,
    key: '',
    run: () => show('export'),
    disabled: !store.selected.length,
  },
  {
    id: 'import',
    label: t('CSV / JSON importieren', 'Import CSV / JSON'),
    icon: FileJson,
    key: '',
    run: () => show('import'),
    disabled: !store.files.length,
  },
  {
    id: 'index',
    label: t('Metadaten für Suche einlesen', 'Read metadata for search'),
    icon: Search,
    key: '',
    run: () => store.indexAll(),
    disabled: !store.files.length,
  },
  {
    id: 'review',
    label: t('Änderungen prüfen', 'Review changes'),
    icon: Check,
    key: '⌘ S',
    run: () => store.reviewChanges(),
    disabled: !store.pendingCount,
  },
  {
    id: 'settings',
    label: t('Darstellung & Sprache', 'Appearance & language'),
    icon: SlidersHorizontal,
    key: '⌘ ,',
    run: () => show('settings'),
    disabled: false,
  },
  {
    id: 'history',
    label: t('Jobs & Wiederherstellung', 'Jobs & recovery'),
    icon: History,
    key: '',
    run: () => show('history'),
    disabled: false,
  },
  {
    id: 'formats',
    label: t('Formatunterstützung & Grenzen', 'Format support & limits'),
    icon: Layers,
    key: '',
    run: () => show('formats'),
    disabled: false,
  },
]);
const filteredActions = computed(() =>
  actions.value.filter((action) =>
    action.label.toLowerCase().includes(commandQuery.value.toLowerCase()),
  ),
);
const title = computed(
  () =>
    (
      ({
        edit: t('Metadaten bearbeiten', 'Edit metadata'),
        privacy: t('Datenschutz prüfen', 'Review privacy'),
        review: t('Änderungen prüfen', 'Review changes'),
        compare: t('Dateien vergleichen', 'Compare files'),
        settings: t('Darstellung & Sprache', 'Appearance & language'),
        columns: t('Ansicht anpassen', 'Customize view'),
        actions: t('Aktionen', 'Actions'),
        palette: t('Befehlspalette', 'Command palette'),
        history: t('Jobs & Wiederherstellung', 'Jobs & recovery'),
        export: t('Metadaten exportieren', 'Export metadata'),
        import: t('Metadaten importieren', 'Import metadata'),
        time: t('Aufnahmezeiten verschieben', 'Shift capture times'),
        rename: t('Stapelumbenennung', 'Batch rename'),
        presets: t('Vorlagen', 'Presets'),
        transfer: t('Metadaten übertragen', 'Transfer metadata'),
        formats: t('Formate & Grenzen', 'Formats & limits'),
        discard: t(
          'Ungespeicherte Änderungen verwerfen',
          'Discard unsaved changes',
        ),
      }) as Record<string, string>
    )[store.dialog] ?? 'Tagryn',
);
const wide = computed(() =>
  ['compare', 'review', 'import', 'history', 'formats'].includes(store.dialog),
);
const importHeaders = computed(() => Object.keys(rows.value[0] ?? {}));
const renameItems = computed(() =>
  store.selectedDocuments.map((doc, index) => {
    try {
      return {
        before: doc.file.name,
        after: renamePreview(renameRule.value, doc, index + 1),
        error: '',
      };
    } catch (e) {
      return { before: doc.file.name, after: '', error: String(e) };
    }
  }),
);
watch(
  () => store.dialog,
  (name) => {
    error.value = '';
    commandQuery.value = '';
    reviewIndex.value = 0;
    if (name === 'edit') {
      enabled.value = store.contextId ? [store.contextId] : [];
      values.value = Object.fromEntries(
        fields.map((field) => {
          const value = aggregate(store.selectedDocuments, field.id).value;
          return [
            field.id,
            Array.isArray(value) ? value.map(text).join('\n') : text(value),
          ];
        }),
      );
      modes.value = {};
      sync.value = false;
      sidecar.value = store.current?.capabilities.write === 'sidecar';
    }
    if (name === 'transfer') sourceId.value = store.selected[0] ?? '';
  },
);
function close() {
  store.dialog = '';
  store.contextId = '';
}
function show(name: string) {
  store.dialog = name;
}
function runAction(action: (typeof actions.value)[number]) {
  if (!action.disabled) {
    close();
    void action.run();
  }
}
function touch(id: string) {
  if (!enabled.value.includes(id)) enabled.value.push(id);
}
function editsFromForm(): Edit[] {
  return fields
    .filter((field) => enabled.value.includes(field.id))
    .map((field) => {
      const input = values.value[field.id] ?? '';
      const value: Json =
        field.type === 'list'
          ? input
              .split('\n')
              .map((v) => v.trim())
              .filter(Boolean)
          : input === ''
            ? null
            : ['number', 'rating'].includes(field.type)
              ? Number(input)
              : input;
      if (typeof value === 'number' && !Number.isFinite(value))
        throw new Error(t('Ungültige Zahl', 'Invalid number'));
      return {
        field: field.id,
        value,
        mode: modes.value[field.id] ?? 'replace',
      };
    });
}
async function stageEdits() {
  try {
    await store.draft(editsFromForm(), undefined, {
      syncLegacy: sync.value,
      forceSidecar: sidecar.value,
    });
    close();
    store.inspectorTab = 'changes';
  } catch (e) {
    error.value = String(e);
  }
}
async function stagePrivacy() {
  await store.draft([], undefined, { privacy: privacy.value });
  close();
  await store.reviewChanges();
}
async function stageRename() {
  error.value = '';
  if (
    renameRule.value
      .match(/\{[^}]+\}/g)
      ?.some((token) => !/^\{(name|date|ext|seq(?::\d)?)\}$/.test(token))
  ) {
    error.value = t('Unbekannte Variable', 'Unknown variable');
    return;
  }
  if (renameItems.value.some((item) => item.error)) {
    error.value = t(
      'Fehlende Aufnahmezeit. Andere Regel wählen.',
      'Missing capture time. Choose another rule.',
    );
    return;
  }
  if (
    new Set(renameItems.value.map((item) => item.after.toLowerCase())).size !==
    renameItems.value.length
  ) {
    error.value = t(
      'Namenskollision innerhalb der Auswahl',
      'Name collision within selection',
    );
    return;
  }
  try {
    const ids = [...store.selected];
    const names = new Set<string>();
    const planned: { id: string; name: string }[] = [];
    for (let i = 0; i < ids.length; i++) {
      const id = ids[i]!;
      await store.load(id);
      const doc = store.documents.get(id);
      if (!doc) throw new Error(store.errors[id] ?? 'Metadata unavailable');
      const name = renamePreview(renameRule.value, doc, i + 1);
      const destination = `${doc.file.folder}/${name}`.toLowerCase();
      if (names.has(destination)) throw new Error(`Rename collision: ${name}`);
      names.add(destination);
      planned.push({ id, name });
    }
    for (const item of planned)
      await store.draft([], [item.id], { rename: item.name });
  } catch (e) {
    error.value = String(e);
    return;
  }
  close();
  await store.reviewChanges();
}
async function stageTime() {
  await store.draft(
    [{ field: 'dateTaken', value: Number(shiftSeconds.value), mode: 'shift' }],
    undefined,
    { syncLegacy: sync.value },
  );
  close();
  await store.reviewChanges();
}
async function transfer() {
  const source = store.documents.get(sourceId.value);
  if (!source) return;
  const edits: Edit[] = transferFields.value.map((id) => ({
    field: id,
    value: fieldValue(source, id),
    mode: 'replace',
  }));
  await store.draft(
    edits,
    store.selected.filter((id) => id !== sourceId.value),
  );
  close();
  await store.reviewChanges();
}
function savePreset() {
  const edits =
    store.dialog === 'edit'
      ? editsFromForm()
      : (Object.values(store.drafts)[0]?.edits ?? []);
  if (!newName.value.trim() || !edits.length) {
    error.value = t(
      'Name und mindestens eine geplante Feldänderung erforderlich.',
      'A name and at least one planned field edit are required.',
    );
    return;
  }
  store.preferences.presets.push({
    id: crypto.randomUUID(),
    name: newName.value.trim(),
    edits,
  });
  newName.value = '';
}
async function applyPreset(preset: Preset) {
  await store.draft(preset.edits);
  close();
}
async function chooseImport() {
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'CSV / JSON', extensions: ['csv', 'json'] }],
    });
    if (!path || Array.isArray(path)) return;
    rows.value = importRows(await api('read_import', { path }));
    importName.value = path.split(/[\\/]/).at(-1) ?? path;
    mapping.value = Object.fromEntries(
      Object.keys(rows.value[0] ?? {}).map((key) => [
        key,
        fields.find((field) => field.id === key || field.tag === key)?.id ?? '',
      ]),
    );
    fileColumn.value =
      Object.keys(rows.value[0] ?? {}).find((key) =>
        ['file', 'filename', 'path', 'SourceFile'].includes(key),
      ) ?? '';
  } catch (e) {
    error.value = String(e);
  }
}
async function stageImport() {
  try {
    const mapped = mapImport(
      rows.value,
      fileColumn.value,
      mapping.value,
      store.files,
    );
    for (const row of mapped) await store.draft(row.edits, [row.fileId]);
    close();
    await store.reviewChanges();
  } catch (e) {
    error.value = String(e);
  }
}
async function restore(jobId: string, fileId: string) {
  try {
    const yes = await confirm(
      t(
        'Die Datei wird aus dem Backup wiederhergestellt. Der aktuelle Stand wird zusätzlich gesichert. Fortfahren?',
        'Restore this file from its backup? The current version will also be preserved.',
      ),
      { title: 'Tagryn', kind: 'warning' },
    );
    if (yes) {
      store.notice = await api<string>('restore_backup', { jobId, fileId });
      await store.load(fileId, true);
    }
  } catch (e) {
    error.value = String(e);
  }
}
function saveView() {
  if (!newName.value.trim()) return;
  store.preferences.views.push({
    name: newName.value.trim(),
    search: store.search,
    filter: store.filter,
    columns: [...store.preferences.columns],
    sort: store.sort,
  });
  newName.value = '';
}
const formatRows = computed(() => [
  [
    'JPEG / TIFF',
    t('Ja', 'Yes'),
    'XMP · EXIF/IPTC ' + t('optional', 'optional'),
    t('Ja', 'Yes'),
  ],
  ['PNG / WebP', t('Ja', 'Yes'), 'XMP', t('Ja', 'Yes')],
  [
    'HEIC / HEIF / AVIF',
    t('Ja', 'Yes'),
    'XMP',
    t('macOS: systemabhängig', 'macOS: system dependent'),
  ],
  [
    'DNG / CR2 / CR3 / NEF / ARW / RAF / ORF / RW2 / PEF',
    t('Ja', 'Yes'),
    'XMP sidecar',
    t('Eingebettetes JPEG, sofern vorhanden', 'Embedded JPEG when available'),
  ],
  ['XMP', t('Ja', 'Yes'), 'XMP', '—'],
  [
    'Video / Audio / PDF',
    t('Engineabhängig', 'Engine dependent'),
    t('In Tagryn schreibgeschützt', 'Read-only in Tagryn'),
    t('Noch nicht implementiert', 'Not implemented yet'),
  ],
]);
const sourceDocument = computed<MetadataDocument | undefined>(() =>
  store.documents.get(sourceId.value),
);
</script>
<template>
  <Modal
    v-if="store.dialog"
    :key="store.dialog"
    :title="title"
    :wide="wide"
    @close="close"
  >
    <p v-if="error" class="inline-error" role="alert">
      <AlertTriangle :size="16" />{{ error }}
    </p>
    <template v-if="['palette', 'actions'].includes(store.dialog)"
      ><label
        v-if="store.dialog === 'palette'"
        class="search-field palette-search"
        ><Search :size="19" /><input
          v-model="commandQuery"
          autofocus
          :placeholder="t('Was möchtest du tun?', 'What would you like to do?')"
          :aria-label="t('Befehl suchen', 'Search commands')"
          @keydown.enter.prevent.stop="
            filteredActions.find((action) => !action.disabled) &&
            runAction(filteredActions.find((action) => !action.disabled)!)
          "
      /></label>
      <div class="command-list">
        <button
          v-for="action in filteredActions"
          :key="action.id"
          :disabled="action.disabled"
          @click="runAction(action)"
        >
          <component :is="action.icon" :size="17" /><span>{{
            action.label
          }}</span
          ><kbd>{{ action.key }}</kbd>
        </button>
      </div></template
    >
    <template v-else-if="store.dialog === 'edit'">
      <div class="dialog-intro">
        <Layers :size="18" /><span
          >{{ store.selected.length }}
          {{
            t(
              'Dateien · Nur aktivierte Felder ändern',
              'files · Only checked fields will change',
            )
          }}</span
        >
      </div>
      <form id="edit-form" class="edit-form" @submit.prevent="stageEdits">
        <div v-for="field in fields" :key="field.id" class="edit-field">
          <label class="edit-label"
            ><input
              v-model="enabled"
              type="checkbox"
              :value="field.id"
            /><span>{{ de ? field.de : field.en }}</span
            ><code>{{ field.tag }}</code></label
          >
          <div v-if="['list', 'textarea'].includes(field.type)">
            <textarea
              v-model="values[field.id]"
              :rows="field.type === 'list' ? 2 : 3"
              :aria-label="de ? field.de : field.en"
              :placeholder="
                aggregate(store.selectedDocuments, field.id).state === 'mixed'
                  ? t('Unterschiedliche Werte', 'Mixed values')
                  : t('Nicht gesetzt', 'Not set')
              "
              @input="touch(field.id)"
            />
            <div v-if="field.type === 'list'" class="list-options">
              <span>{{ t('Ein Eintrag pro Zeile', 'One item per line') }}</span
              ><select
                v-model="modes[field.id]"
                :aria-label="t('Listenoperation', 'List operation')"
                @change="touch(field.id)"
              >
                <option :value="undefined">
                  {{ t('Ersetzen', 'Replace') }}
                </option>
                <option value="add">{{ t('Hinzufügen', 'Add') }}</option>
                <option value="remove">
                  {{ t('Gezielt entfernen', 'Remove matching') }}
                </option>
              </select>
            </div>
          </div>
          <select
            v-else-if="field.type === 'rating'"
            v-model="values[field.id]"
            :aria-label="t('Bewertung', 'Rating')"
            @change="touch(field.id)"
          >
            <option value="">{{ t('Nicht gesetzt', 'Not set') }}</option>
            <option value="-1">{{ t('Abgelehnt', 'Rejected') }}</option>
            <option
              v-for="rating in [0, 1, 2, 3, 4, 5]"
              :key="rating"
              :value="String(rating)"
            >
              {{ rating }} {{ '★'.repeat(rating) }}
            </option></select
          ><input
            v-else
            v-model="values[field.id]"
            :type="field.type === 'number' ? 'number' : 'text'"
            step="any"
            :aria-label="de ? field.de : field.en"
            :placeholder="
              field.type === 'date'
                ? 'YYYY:MM:DD HH:mm:ss±HH:mm'
                : aggregate(store.selectedDocuments, field.id).state === 'mixed'
                  ? t('Unterschiedliche Werte', 'Mixed values')
                  : t('Nicht gesetzt', 'Not set')
            "
            @input="touch(field.id)"
          />
        </div>
        <div class="write-policy">
          <label class="checkbox-label"
            ><input
              v-model="sidecar"
              type="checkbox"
              :disabled="
                store.selectedDocuments.some(
                  (doc) => doc.capabilities.write === 'sidecar',
                )
              "
            />{{
              t(
                'In XMP-Sidecar schreiben (RAW: Standard)',
                'Write to XMP sidecar (RAW default)',
              )
            }}</label
          ><label class="checkbox-label"
            ><input v-model="sync" type="checkbox" :disabled="sidecar" />{{
              t(
                'EXIF/IPTC ausdrücklich synchronisieren (JPEG/TIFF)',
                'Explicitly synchronize EXIF/IPTC (JPEG/TIFF)',
              )
            }}</label
          >
          <p>
            {{
              t(
                'Standard: XMP. Vorhandene Zeitzonen bleiben erhalten; fehlende werden nicht ergänzt. GPS-Felder werden in XMP geschrieben.',
                'Default: XMP. Existing timezones are preserved; missing zones are not inferred. GPS fields are written to XMP.',
              )
            }}
          </p>
        </div>
      </form>
      <details class="advanced">
        <summary>{{ t('Als Vorlage speichern', 'Save as preset') }}</summary>
        <div class="inline-form">
          <input
            v-model="newName"
            :placeholder="t('Name der Vorlage', 'Preset name')"
            :aria-label="t('Name der Vorlage', 'Preset name')"
          /><button
            class="button"
            :disabled="!enabled.length"
            @click="savePreset"
          >
            {{ t('Vorlage sichern', 'Save preset') }}
          </button>
        </div>
      </details>
    </template>
    <template v-else-if="store.dialog === 'privacy'"
      ><p class="dialog-copy">
        {{
          t(
            'Wähle, welche Informationen entfernt werden sollen. Die nächste Ansicht zeigt alle betroffenen Felder und Gruppen vor der Ausführung.',
            'Choose which information to remove. The next view shows every affected field and group before execution.',
          )
        }}
      </p>
      <div class="privacy-presets">
        <label
          v-for="preset in [
            {
              id: 'location',
              icon: MapPin,
              title: t('Standort entfernen', 'Remove location'),
              description: t(
                'GPS, Ortsangaben und mögliche Standortspuren',
                'GPS, place names and potential location traces',
              ),
            },
            {
              id: 'personal',
              icon: LockKeyhole,
              title: t(
                'Persönliche Angaben entfernen',
                'Remove personal information',
              ),
              description: t(
                'Standort, Urheber, Kontakt und Geräteseriennummern',
                'Location, creator, contact and device serial numbers',
              ),
            },
            {
              id: 'publication',
              icon: ShieldCheck,
              title: t(
                'Für Veröffentlichung vorbereiten',
                'Prepare for publication',
              ),
              description: t(
                'Persönliche Daten, Beschreibungen, Keywords und Verlauf',
                'Personal information, descriptions, keywords and history',
              ),
            },
          ]"
          :key="preset.id"
          :class="{ selected: privacy === preset.id }"
          ><input v-model="privacy" type="radio" :value="preset.id" /><component
            :is="preset.icon"
            :size="23"
          /><span
            ><strong>{{ preset.title }}</strong
            ><small>{{ preset.description }}</small></span
          ></label
        >
      </div>
      <div class="policy-callout">
        <ShieldCheck :size="19" />
        <p>
          {{
            t(
              'Orientierung und Farbprofile bleiben erhalten. MakerNotes und eingebettete Vorschaubilder werden, falls vorhanden, zur Entfernung eingeplant. Ergebnisse werden erneut geprüft.',
              'Orientation and color profiles are preserved. MakerNotes and embedded previews, when present, are included in the removal plan. Results are checked again.',
            )
          }}
        </p>
      </div>
      <p class="help-text">
        {{
          t(
            'Keine Garantie vollständiger Anonymisierung. RAW, Video, Audio und PDF: eingebettete Bereinigung in dieser Version nicht unterstützt. Vorhandene Sidecars getrennt prüfen.',
            'No guarantee of complete anonymization. Embedded cleanup of RAW, video, audio and PDF is unsupported in this version. Check existing sidecars separately.',
          )
        }}
      </p></template
    >
    <template v-else-if="store.dialog === 'review' && store.plan"
      ><div class="review-summary">
        <ShieldCheck :size="24" />
        <div>
          <strong
            >{{ planCount }} {{ t('Änderungen', 'changes') }} ·
            {{ store.plan.files.length }} {{ t('Dateien', 'files') }}</strong
          >
          <p>
            {{
              t(
                'Originale werden gesichert. Jede Datei wird erneut gelesen und geprüft.',
                'Originals are backed up. Each file is reread and verified.',
              )
            }}
          </p>
        </div>
        <span v-if="planErrors" class="warning-label"
          >{{ planErrors }} {{ t('nicht ausführbar', 'not executable') }}</span
        >
      </div>
      <label class="stacked-label"
        >{{ t('Datei prüfen', 'Review file')
        }}<select v-model="reviewIndex">
          <option
            v-for="(file, index) in store.plan.files"
            :key="file.file.id"
            :value="index"
          >
            {{ file.error ? '⚠ ' : '' }}{{ file.file.name }} ·
            {{ file.differences.length }} {{ t('Änderungen', 'changes') }}
          </option>
        </select></label
      ><template v-if="currentPlan"
        ><div v-if="currentPlan.error" class="inline-error">
          {{ currentPlan.error }}
        </div>
        <div class="review-target">
          <span>{{ t('Schreibziel', 'Write target') }}</span
          ><code>{{ currentPlan.renameTo || currentPlan.target }}</code>
        </div>
        <div class="diff-table-scroll">
          <table class="data-table diff-table">
            <thead>
              <tr>
                <th>Tag</th>
                <th>{{ t('Vorher', 'Before') }}</th>
                <th>{{ t('Nachher', 'After') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(difference, index) in currentPlan.differences"
                :key="index"
              >
                <td>
                  <code>{{ difference.tag }}</code
                  ><small>{{ difference.source }}</small>
                </td>
                <td class="before">{{ text(difference.before) || '∅' }}</td>
                <td class="after">
                  {{ text(difference.after) || t('Entfernen', 'Remove') }}
                </td>
              </tr>
            </tbody>
          </table>
          <p
            v-if="!currentPlan.differences.length && !currentPlan.error"
            class="help-text"
          >
            {{ t('Keine Änderung nötig', 'No changes needed') }}
          </p>
        </div>
        <div class="review-warnings">
          <p v-for="warning in currentPlan.warnings" :key="warning">
            <AlertTriangle :size="14" />{{ warning }}
          </p>
        </div></template
      >
      <p class="help-text">
        {{
          t(
            'Batch-Jobs sind nicht global atomar. Fehler betreffen einzelne Dateien. Abbruch startet keine weiteren Dateien.',
            'Batch jobs are not globally atomic. Errors affect individual files. Cancellation starts no further files.',
          )
        }}
      </p></template
    >
    <Comparison v-else-if="store.dialog === 'compare'" />
    <template v-else-if="store.dialog === 'settings'"
      ><div class="setting-section">
        <h3>{{ t('Darstellung', 'Appearance') }}</h3>
        <div class="theme-options">
          <button
            v-for="theme in [
              { id: 'light' as const, icon: Sun, label: t('Hell', 'Light') },
              { id: 'dark' as const, icon: Moon, label: t('Dunkel', 'Dark') },
              {
                id: 'system' as const,
                icon: Monitor,
                label: t('System', 'System'),
              },
            ]"
            :key="theme.id"
            :class="{ selected: store.preferences.theme === theme.id }"
            :aria-pressed="store.preferences.theme === theme.id"
            @click="store.preferences.theme = theme.id"
          >
            <component :is="theme.icon" :size="23" /><span>{{
              theme.label
            }}</span>
          </button>
        </div>
      </div>
      <label class="setting-row"
        ><span>{{ t('Sprache', 'Language') }}</span
        ><select v-model="store.preferences.language">
          <option value="de">Deutsch</option>
          <option value="en">English</option>
        </select></label
      ><label class="setting-row"
        ><span>{{ t('Schriftgröße', 'Text size') }}</span
        ><input
          v-model.number="store.preferences.fontSize"
          type="range"
          min="12"
          max="17"
        /><span>{{ store.preferences.fontSize }} px</span></label
      ><label class="setting-row"
        ><span>{{ t('Technische Tag-Namen', 'Technical tag names') }}</span
        ><input v-model="store.preferences.technical" type="checkbox" /></label
      ><label class="setting-row"
        ><span>{{ t('Unterordner einlesen', 'Include subfolders') }}</span
        ><input v-model="store.preferences.recursive" type="checkbox"
      /></label>
      <div class="setting-section">
        <h3>
          <Keyboard :size="15" />{{ t('Tastenkürzel', 'Keyboard shortcuts') }}
        </h3>
        <div class="shortcut-list">
          <span>{{ t('Dateien öffnen', 'Open files') }}<kbd>⌘/Ctrl O</kbd></span
          ><span
            >{{ t('Befehlspalette', 'Command palette')
            }}<kbd>⌘/Ctrl K</kbd></span
          ><span
            >{{ t('Änderungen prüfen', 'Review changes')
            }}<kbd>⌘/Ctrl S</kbd></span
          ><span
            >{{ t('Bearbeiten / Vergleichen', 'Edit / Compare')
            }}<kbd>⌘/Ctrl E / D</kbd></span
          ><span
            >{{ t('Suche / Einstellungen', 'Search / Settings')
            }}<kbd>⌘/Ctrl F / ,</kbd></span
          >
        </div>
      </div>
      <p class="help-text">
        Tagryn 0.1.0 · ExifTool {{ store.engine || '—' }}<br />{{
          t(
            'Offline. Keine Telemetrie, keine automatischen Kartenabfragen.',
            'Offline. No telemetry, no automatic map requests.',
          )
        }}
      </p></template
    >
    <template v-else-if="store.dialog === 'columns'"
      ><p class="help-text">
        {{
          t(
            'Zusätzliche Metadatenspalten werden nach dem Einlesen verfügbar.',
            'Additional metadata columns become available after reading metadata.',
          )
        }}
      </p>
      <label
        v-for="column in [
          { id: 'type', label: t('Dateityp', 'File type') },
          { id: 'size', label: t('Größe', 'Size') },
          { id: 'rating', label: t('Bewertung', 'Rating') },
          { id: 'camera', label: t('Kamera', 'Camera') },
          { id: 'date', label: t('Aufnahmezeit', 'Capture date') },
          { id: 'modified', label: t('Geändert', 'Modified') },
        ]"
        :key="column.id"
        class="setting-row"
        ><span>{{ column.label }}</span
        ><input
          v-model="store.preferences.columns"
          type="checkbox"
          :value="column.id"
      /></label>
      <label class="stacked-label"
        >{{
          t('Eigenes technisches Tag als Spalte', 'Custom technical tag column')
        }}
        <select
          aria-label="Technical tag column"
          @change="
            (event) => {
              const value = (event.target as HTMLSelectElement).value;
              if (value && !store.preferences.columns.includes(value))
                store.preferences.columns.push(value);
              (event.target as HTMLSelectElement).value = '';
            }
          "
        >
          <option value="">{{ t('Tag wählen …', 'Choose tag …') }}</option>
          <option
            v-for="tag in store.current?.tags.filter((tag) => !tag.derived) ??
            []"
            :key="tag.id"
            :value="`tag:${tag.location}:${tag.name}`"
          >
            {{ tag.location }}:{{ tag.name }}
          </option>
        </select>
      </label>
      <label
        v-for="column in store.preferences.columns.filter((column) =>
          column.startsWith('tag:'),
        )"
        :key="column"
        class="setting-row"
        ><code>{{ column.slice(4) }}</code
        ><button
          class="text-button"
          @click="
            store.preferences.columns = store.preferences.columns.filter(
              (value) => value !== column,
            )
          "
        >
          {{ t('Entfernen', 'Remove') }}
        </button></label
      >
      <div class="setting-section">
        <h3>{{ t('Aktuelle Ansicht speichern', 'Save current view') }}</h3>
        <div class="inline-form">
          <input
            v-model="newName"
            :placeholder="t('Name der Ansicht', 'View name')"
            :aria-label="t('Name der Ansicht', 'View name')"
          /><button
            class="button"
            :disabled="!newName.trim()"
            @click="saveView"
          >
            {{ t('Speichern', 'Save') }}
          </button>
        </div>
      </div></template
    >
    <template v-else-if="store.dialog === 'history'"
      ><div v-if="!store.jobs.length" class="modal-empty">
        <History :size="30" />
        <p>{{ t('Noch keine Jobs.', 'No jobs yet.') }}</p>
      </div>
      <section v-for="job in store.jobs" :key="job.id" class="history-job">
        <header>
          <strong>{{
            job.kind === 'scan'
              ? t('Dateien einlesen', 'Read files')
              : t('Metadaten speichern', 'Save metadata')
          }}</strong
          ><span class="status-pill">{{ job.status }}</span
          ><span
            >{{ job.completed }}/{{ job.total }} · {{ job.errors }}
            {{ t('Fehler', 'errors') }}</span
          ><button
            v-if="['running', 'queued'].includes(job.status)"
            class="button small"
            @click="api('cancel_job', { jobId: job.id }).catch(store.report)"
          >
            {{ t('Abbrechen', 'Cancel') }}
          </button>
        </header>
        <small>{{
          new Date(job.started).toLocaleString(de ? 'de-AT' : 'en-US')
        }}</small>
        <details v-if="job.results.length">
          <summary>
            {{ t('Dateiergebnisse und Backups', 'File results and backups') }}
          </summary>
          <div
            v-for="result in job.results"
            :key="result.fileId || result.path"
            class="history-result"
          >
            <div>
              <strong>{{ result.path.split(/[\\/]/).at(-1) }}</strong
              ><span class="status-pill">{{ result.status }}</span>
            </div>
            <p>{{ result.message }}</p>
            <p
              v-for="warning in result.warnings"
              :key="warning"
              class="help-text"
            >
              {{ warning }}
            </p>
            <code v-if="result.backup">{{ result.backup }}</code
            ><button
              v-if="result.after && result.status !== 'restored'"
              class="button small"
              @click="restore(job.id, result.fileId)"
            >
              <History :size="13" />{{
                t('Gespeicherte Datei wiederherstellen', 'Restore saved file')
              }}
            </button>
          </div>
        </details>
      </section></template
    >
    <template v-else-if="store.dialog === 'export'"
      ><p class="dialog-copy">
        {{ store.selected.length }}
        {{
          t(
            'Dateien mit Gruppen, Instanzen, Herkunft, Rohwerten und formatierten Werten exportieren.',
            'files with groups, instances, source, raw values and formatted values.',
          )
        }}
      </p>
      <div class="export-options">
        <button
          class="button"
          @click="
            store.exportFiles('json');
            close();
          "
        >
          <FileJson :size="20" /><span
            ><strong>JSON</strong
            ><small>{{
              t('Vollständiges Metadatenmodell', 'Complete metadata model')
            }}</small></span
          ><Download :size="16" /></button
        ><button
          class="button"
          @click="
            store.exportFiles('csv');
            close();
          "
        >
          <Download :size="20" /><span
            ><strong>CSV</strong
            ><small>{{
              t(
                'Eine Zeile pro Tag · Excel-Formeln entschärft',
                'One row per tag · Spreadsheet formulas escaped',
              )
            }}</small></span
          ><Download :size="16" />
        </button>
      </div>
      <p class="help-text">
        {{
          t(
            'Ein neuer Dateiname schützt bestehende Exporte vor Überschreiben.',
            'A new filename protects existing exports from being overwritten.',
          )
        }}
      </p></template
    >
    <template v-else-if="store.dialog === 'import'"
      ><button class="button" @click="chooseImport">
        <FolderOpen :size="16" />{{
          t('CSV oder JSON wählen', 'Choose CSV or JSON')
        }}
      </button>
      <p v-if="importName" class="help-text">
        {{ importName }} · {{ rows.length }} {{ t('Zeilen', 'rows') }}
      </p>
      <template v-if="rows.length"
        ><label class="stacked-label"
          >{{
            t(
              'Spalte für Dateipfad oder eindeutigen Dateinamen',
              'Column containing file path or unique filename',
            )
          }}<select v-model="fileColumn">
            <option value="">{{ t('Wählen …', 'Choose …') }}</option>
            <option v-for="header in importHeaders" :key="header">
              {{ header }}
            </option>
          </select></label
        >
        <div class="mapping-grid">
          <label
            v-for="header in importHeaders.filter((h) => h !== fileColumn)"
            :key="header"
            ><code>{{ header }}</code
            ><ArrowRight :size="14" /><select v-model="mapping[header]">
              <option value="">{{ t('Ignorieren', 'Ignore') }}</option>
              <option v-for="field in fields" :key="field.id" :value="field.id">
                {{ de ? field.de : field.en }}
              </option>
            </select></label
          >
        </div>
        <p class="help-text">
          {{
            t(
              'Leere zugeordnete Zellen entfernen Werte. Dateinamen müssen zu genau einer bereits geöffneten Datei passen. Listen: JSON-Array oder kommagetrennte Werte.',
              'Empty mapped cells remove values. Filenames must match exactly one already-opened file. Lists: JSON arrays or comma-separated values.',
            )
          }}
        </p>
        <div class="import-preview">
          <strong>{{ t('Erste Zeile', 'First row') }}</strong>
          <pre>{{ JSON.stringify(rows[0], null, 2) }}</pre>
        </div></template
      ></template
    >
    <template v-else-if="store.dialog === 'time'"
      ><p class="dialog-copy">
        {{
          t(
            'Kamerauhren um dieselbe Zeitspanne korrigieren. Vorhandene Zeitzonen werden beibehalten. Ohne Zeitzone bleibt die Zeit lokal und unbestimmt.',
            'Correct camera clocks by the same interval. Existing timezone offsets are preserved. Without an offset, times remain local and unspecified.',
          )
        }}
      </p>
      <label class="stacked-label"
        >{{
          t(
            'Verschiebung in Sekunden (negativ = früher)',
            'Shift in seconds (negative = earlier)',
          )
        }}<input v-model.number="shiftSeconds" type="number" step="1"
      /></label>
      <div class="quick-shifts">
        <button
          v-for="amount in [-3600, -60, 60, 3600]"
          :key="amount"
          class="button small"
          @click="shiftSeconds += amount"
        >
          {{ amount > 0 ? '+' : '' }}{{ amount }} s
        </button>
      </div>
      <label class="checkbox-label"
        ><input v-model="sync" type="checkbox" />{{
          t(
            'EXIF-Aufnahmezeit mit synchronisieren (JPEG/TIFF)',
            'Also synchronize EXIF capture time (JPEG/TIFF)',
          )
        }}</label
      >
      <p class="help-text">
        {{
          t(
            'GPX-Zuordnung ist noch nicht implementiert. Koordinaten können im Editor eingegeben werden.',
            'GPX matching is not implemented yet. Coordinates can be entered in the editor.',
          )
        }}
      </p></template
    >
    <template v-else-if="store.dialog === 'rename'"
      ><label class="stacked-label"
        >{{ t('Namensregel', 'Filename rule')
        }}<input v-model="renameRule" type="text" spellcheck="false"
      /></label>
      <p class="help-text">
        <code>{name} · {date} · {seq:3} · {ext}</code><br />{{
          t(
            'Die Dateiendung muss erhalten bleiben. Vorhandene Ziele und doppelte Namen werden geprüft.',
            'The extension must be preserved. Existing destinations and duplicate names are checked.',
          )
        }}
      </p>
      <div class="rename-preview">
        <div v-for="item in renameItems" :key="item.before">
          <span>{{ item.before }}</span
          ><ArrowRight :size="13" /><strong
            :class="{ 'error-text': item.error }"
            >{{ item.error || item.after }}</strong
          >
        </div>
      </div>
      <p class="help-text">
        {{
          t(
            'Dateien mit vorhandenen Sidecars: gemeinsame Umbenennung noch nicht freigegeben. Der Plan zeigt diese Dateien als nicht ausführbar.',
            'Files with existing sidecars: paired renaming is not enabled yet. The plan marks those files as not executable.',
          )
        }}
      </p></template
    >
    <template v-else-if="store.dialog === 'transfer'"
      ><label class="stacked-label"
        >{{ t('Quelle', 'Source')
        }}<select v-model="sourceId">
          <option
            v-for="doc in store.selectedDocuments"
            :key="doc.file.id"
            :value="doc.file.id"
          >
            {{ doc.file.name }}
          </option>
        </select></label
      >
      <p class="help-text">
        {{ store.selected.length - 1 }}
        {{
          t(
            'Zieldateien · Unterstützte XMP-Felder auswählen',
            'target files · Choose supported XMP fields',
          )
        }}
      </p>
      <label v-for="field in fields" :key="field.id" class="transfer-field"
        ><input
          v-model="transferFields"
          type="checkbox"
          :value="field.id"
        /><span>{{ de ? field.de : field.en }}</span
        ><code>{{
          text(fieldValue(sourceDocument, field.id)) || '∅'
        }}</code></label
      >
      <p class="help-text">
        {{
          t(
            'Fehlende Quellwerte werden als Entfernung eingeplant. MakerNotes, ICC und beliebige technische Gruppen werden nicht übertragen.',
            'Missing source values are planned as removals. MakerNotes, ICC and arbitrary technical groups are not transferred.',
          )
        }}
      </p></template
    >
    <template v-else-if="store.dialog === 'presets'"
      ><div v-if="!store.preferences.presets.length" class="modal-empty">
        <SlidersHorizontal :size="28" />
        <p>
          {{
            t(
              'Noch keine Vorlagen. Im Editor Felder aktivieren und als Vorlage speichern.',
              'No presets yet. Enable fields in the editor and save them as a preset.',
            )
          }}
        </p>
      </div>
      <div
        v-for="preset in store.preferences.presets"
        :key="preset.id"
        class="preset-row"
      >
        <div>
          <strong>{{ preset.name }}</strong
          ><small>{{ preset.edits.length }} {{ t('Felder', 'fields') }}</small>
        </div>
        <button
          class="button small"
          :disabled="!store.selected.length"
          @click="applyPreset(preset)"
        >
          {{ t('Anwenden', 'Apply') }}</button
        ><button
          class="icon-button"
          :aria-label="t('Vorlage löschen', 'Delete preset')"
          @click="
            store.preferences.presets = store.preferences.presets.filter(
              (p) => p.id !== preset.id,
            )
          "
        >
          <Trash2 :size="14" />
        </button></div
    ></template>
    <template v-else-if="store.dialog === 'formats'"
      ><p class="dialog-copy">
        {{
          t(
            'Lesen, Schreiben und Vorschau sind getrennte Fähigkeiten. Eine fehlende Vorschau blockiert die Analyse nicht.',
            'Reading, writing and previewing are independent capabilities. Missing previews do not block analysis.',
          )
        }}
      </p>
      <div class="table-scroll">
        <table class="data-table">
          <thead>
            <tr>
              <th>Format</th>
              <th>{{ t('Lesen', 'Read') }}</th>
              <th>{{ t('Schreiben', 'Write') }}</th>
              <th>{{ t('Vorschau', 'Preview') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in formatRows" :key="row[0]">
              <td v-for="(cell, index) in row" :key="index">{{ cell }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="limitations">
        <h3>
          {{
            t(
              'Transparente Grenzen dieser Version',
              'Known limitations in this version',
            )
          }}
        </h3>
        <p>
          {{
            t(
              'Bearbeitung ist auf freigegebene Felder begrenzt. Abgeleitete Tags und unbekannte Felder sind schreibgeschützt. Hersteller-RAW wird ausschließlich per Sidecar bearbeitet.',
              'Editing is limited to supported fields. Derived and unknown tags are read-only. Camera RAW is edited through sidecars only.',
            )
          }}
        </p>
        <p>
          {{
            t(
              'Nicht implementiert: GPX-Zuordnung, Onlinekarten, gepaarte Sidecar-Umbenennung, eingebettetes Schreiben von Audio/Video/PDF sowie Farbmanagement für jede Vorschau. Vergleich: maximal acht Dateien gleichzeitig.',
              'Not implemented: GPX matching, online maps, paired sidecar renaming, embedded audio/video/PDF writing, and color management for every preview. Comparison: at most eight files at once.',
            )
          }}
        </p>
        <p>
          {{
            t(
              'Metadatensuche berücksichtigt bisher eingelesene Dateien. Große Ordner lassen sich schrittweise einlesen. Backups enthalten ursprüngliche sensible Daten; bei Weitergabe nicht mitliefern.',
              'Metadata search covers files read so far. Large folders can be indexed progressively. Backups contain original sensitive information; do not include them when sharing.',
            )
          }}
        </p>
        <p>
          {{
            t(
              'Lokaler Entwicklungsstand. Windows/Linux und signierte Installationspakete benötigen ihre jeweiligen Build- und Abnahmeläufe.',
              'Local development build. Windows/Linux and signed installers require their respective build and acceptance runs.',
            )
          }}
        </p>
      </div></template
    >
    <template v-else-if="store.dialog === 'discard'"
      ><p class="dialog-copy">
        {{ store.pendingCount }}
        {{
          t(
            'Dateien haben ungespeicherte Änderungen. Verwerfen ändert keine Datei auf dem Datenträger.',
            'files have unsaved changes. Discarding does not change files on disk.',
          )
        }}
      </p>
      <p class="help-text">
        {{
          t(
            'Bereits gespeicherte Dateien lassen sich unter „Jobs & Wiederherstellung“ aus Backups wiederherstellen.',
            'Already saved files can be restored from backups under “Jobs & recovery”.',
          )
        }}
      </p></template
    >
    <template #footer>
      <template v-if="store.dialog === 'edit'"
        ><span class="help-text"
          >{{ enabled.length }}
          {{ t('Felder aktiviert', 'fields enabled') }}</span
        ><button class="button" @click="close">
          {{ t('Abbrechen', 'Cancel') }}</button
        ><button
          class="button primary"
          form="edit-form"
          type="submit"
          :disabled="!enabled.length || !!store.busy"
        >
          {{ t('Änderungen vormerken', 'Stage changes')
          }}<ArrowRight :size="14" /></button
      ></template>
      <template v-else-if="store.dialog === 'privacy'"
        ><button class="button" @click="close">
          {{ t('Abbrechen', 'Cancel') }}</button
        ><button
          class="button primary"
          :disabled="!!store.busy"
          @click="stagePrivacy"
        >
          {{ t('Entfernung prüfen', 'Review removal')
          }}<ArrowRight :size="14" /></button
      ></template>
      <template v-else-if="store.dialog === 'review'"
        ><span class="help-text"
          ><LockKeyhole :size="12" />{{
            t('Backup immer aktiv', 'Backups always enabled')
          }}</span
        ><button class="button" @click="close">{{ t('Zurück', 'Back') }}</button
        ><button
          class="button primary"
          :disabled="
            !planCount ||
            (store.plan?.files.length ?? 0) === planErrors ||
            !native
          "
          @click="store.execute"
        >
          <ShieldCheck :size="15" />{{
            t('Gesichert speichern', 'Save with backup')
          }}
        </button></template
      >
      <template v-else-if="store.dialog === 'time'"
        ><button class="button" @click="close">
          {{ t('Abbrechen', 'Cancel') }}</button
        ><button
          class="button primary"
          :disabled="!shiftSeconds"
          @click="stageTime"
        >
          {{ t('Verschiebung prüfen', 'Review shift') }}
        </button></template
      >
      <template v-else-if="store.dialog === 'rename'"
        ><button class="button" @click="close">
          {{ t('Abbrechen', 'Cancel') }}</button
        ><button
          class="button primary"
          :disabled="!renameItems.length"
          @click="stageRename"
        >
          {{ t('Namen prüfen', 'Review names') }}
        </button></template
      >
      <template v-else-if="store.dialog === 'transfer'"
        ><button class="button" @click="close">
          {{ t('Abbrechen', 'Cancel') }}</button
        ><button
          class="button primary"
          :disabled="!transferFields.length"
          @click="transfer"
        >
          {{ t('Übertragung prüfen', 'Review transfer') }}
        </button></template
      >
      <template v-else-if="store.dialog === 'import'"
        ><button class="button" @click="close">
          {{ t('Abbrechen', 'Cancel') }}</button
        ><button
          class="button primary"
          :disabled="!rows.length"
          @click="stageImport"
        >
          {{ t('Validieren und prüfen', 'Validate and review') }}
        </button></template
      >
      <template v-else-if="store.dialog === 'discard'"
        ><button class="button" @click="close">
          {{ t('Behalten', 'Keep edits') }}</button
        ><button
          class="button danger"
          @click="
            store.drafts = {};
            store.plan = null;
            close();
          "
        >
          {{
            t('Ungespeicherte Änderungen verwerfen', 'Discard unsaved changes')
          }}
        </button></template
      >
      <button v-else class="button" @click="close">
        {{ t('Fertig', 'Done') }}
      </button>
      <span v-if="review" class="help-text">{{
        t(
          'Browservorschau · Schreiben nur in Desktop-App',
          'Browser preview · Writing requires desktop app',
        )
      }}</span>
    </template>
  </Modal>
</template>
