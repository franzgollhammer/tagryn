<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useVirtualizer } from '@tanstack/vue-virtual';
import {
  Image as ImageIcon,
  Camera,
  MapPin,
  Copyright,
  CalendarDays,
  Pencil,
  Copy,
  Check,
  Search,
  SlidersHorizontal,
  Star,
  LockKeyhole,
  Layers,
  AlertTriangle,
  ArrowRight,
  ScanLine,
  RefreshCw,
  X,
} from '@lucide/vue';
import { useWorkspace } from '../stores/workspace';
import { useI18n } from '../i18n';
import { aggregate, fields } from '../domain/fields';
import { sizeText, tagValue, text } from '../domain/types';
import type { Json, Tag } from '../domain/types';
const store = useWorkspace();
const { t, de } = useI18n();
const tagSearch = ref('');
const group = ref('all');
const tagFilter = ref('all');
const raw = ref(false);
const copied = ref('');
const tagScroll = ref<HTMLElement>();
const tabs = computed(() => [
  { id: 'overview' as const, label: t('Übersicht', 'Overview') },
  { id: 'all' as const, label: t('Alle Metadaten', 'All metadata') },
  { id: 'changes' as const, label: t('Änderungen', 'Changes') },
]);
const current = computed(() => store.current);
const multi = computed(() => store.selected.length > 1);
const groups = computed(() => [
  ...new Set(current.value?.tags.map((tag) => tag.group) ?? []),
]);
const tags = computed(() =>
  (current.value?.tags ?? []).filter((tag) => {
    if (group.value !== 'all' && tag.group !== group.value) return false;
    if (tagFilter.value === 'writable' && !tag.writable) return false;
    if (
      tagFilter.value === 'favorites' &&
      !store.preferences.tagFavorites.includes(`${tag.location}:${tag.name}`)
    )
      return false;
    if (
      tagFilter.value === 'changed' &&
      !store.drafts[current.value?.file.id ?? '']?.edits.some(
        (e) => e.field === tag.field,
      )
    )
      return false;
    return `${tag.key} ${tag.label} ${text(tag.raw)} ${text(tag.formatted)}`
      .toLocaleLowerCase()
      .includes(tagSearch.value.toLocaleLowerCase());
  }),
);
const virtualizer = useVirtualizer(
  computed(() => ({
    count: tags.value.length,
    getScrollElement: () => tagScroll.value ?? null,
    estimateSize: () => (store.preferences.technical ? 85 : 67),
    overscan: 8,
  })),
);
const items = computed(() => virtualizer.value.getVirtualItems());
watch(
  [() => store.preferences.technical, () => store.preferences.fontSize],
  () => virtualizer.value.measure(),
);
const information = computed(() => [
  {
    label: t('Aufgenommen', 'Captured'),
    value: tagValue(current.value, ['DateTimeOriginal']),
    icon: CalendarDays,
  },
  {
    label: t('Kamera', 'Camera'),
    value: tagValue(current.value, ['Model']),
    icon: Camera,
  },
  {
    label: t('Objektiv', 'Lens'),
    value: tagValue(current.value, ['LensModel', 'LensID']),
    icon: ScanLine,
  },
]);
const exposure = computed(() => [
  {
    label: t('Blende', 'Aperture'),
    value: text(tagValue(current.value, ['FNumber']))
      ? `ƒ/${text(tagValue(current.value, ['FNumber']))}`
      : '—',
  },
  {
    label: t('Verschluss', 'Shutter'),
    value: text(tagValue(current.value, ['ExposureTime'])) || '—',
  },
  { label: 'ISO', value: text(tagValue(current.value, ['ISO'])) || '—' },
  {
    label: t('Brennweite', 'Focal length'),
    value: text(tagValue(current.value, ['FocalLength'])) || '—',
  },
]);
const identityFields = computed(() =>
  fields.filter((f) =>
    [
      'title',
      'description',
      'keywords',
      'creator',
      'copyright',
      'rating',
    ].includes(f.id),
  ),
);
const conflicts = computed(() =>
  fields.filter((field) => {
    const embedded = current.value?.tags.find(
      (tag) =>
        `${tag.location}:${tag.name}` === field.tag &&
        tag.source === 'embedded',
    );
    const sidecar = current.value?.tags.find(
      (tag) =>
        `${tag.location}:${tag.name}` === field.tag && tag.source === 'sidecar',
    );
    return (
      embedded &&
      sidecar &&
      JSON.stringify(embedded.raw) !== JSON.stringify(sidecar.raw)
    );
  }),
);
async function copy(value: Json | string, id: string) {
  try {
    await navigator.clipboard.writeText(
      typeof value === 'string' ? value : text(value),
    );
    copied.value = id;
    setTimeout(() => (copied.value = ''), 1400);
  } catch (e) {
    store.report(e);
  }
}
function edit(id: string) {
  store.contextId = id;
  store.dialog = 'edit';
}
function favorite(tag: Tag) {
  const key = `${tag.location}:${tag.name}`;
  const index = store.preferences.tagFavorites.indexOf(key);
  if (index >= 0) store.preferences.tagFavorites.splice(index, 1);
  else store.preferences.tagFavorites.push(key);
}
const statusText = (id: string) => {
  const state = aggregate(store.selectedDocuments, id);
  return state.state === 'mixed'
    ? t('Unterschiedliche Werte', 'Mixed values')
    : state.state === 'partial'
      ? `${text(state.value) || t('Unterschiedlich', 'Mixed')} · ${state.missing} ${t('fehlen', 'missing')}`
      : text(state.value) || t('Nicht gesetzt', 'Not set');
};
</script>
<template>
  <aside
    class="inspector"
    :aria-label="t('Metadaten-Inspector', 'Metadata inspector')"
  >
    <header class="inspector-header">
      <span>{{ t('Inspector', 'Inspector') }}</span>
      <div class="inline-actions">
        <button
          class="icon-button"
          :disabled="!current"
          :aria-label="t('Neu einlesen', 'Reload metadata')"
          :title="t('Neu einlesen', 'Reload metadata')"
          @click="current && store.load(current.file.id, true)"
        >
          <RefreshCw :size="14" /></button
        ><button
          class="icon-button"
          :class="{ active: store.preferences.technical }"
          :aria-pressed="store.preferences.technical"
          :aria-label="t('Technische Tags anzeigen', 'Show technical tags')"
          :title="t('Technische Tags anzeigen', 'Show technical tags')"
          @click="store.preferences.technical = !store.preferences.technical"
        >
          <SlidersHorizontal :size="15" />
        </button>
      </div>
    </header>
    <div v-if="!store.selected.length" class="inspector-empty">
      <Layers :size="30" :stroke-width="1" />
      <p>
        {{ t('Die Details liegen im Bild.', 'The details are in the image.') }}
      </p>
      <span>{{ t('Wähle eine Datei aus.', 'Select a file to inspect.') }}</span>
      <div class="inspector-promises">
        <span
          ><ScanLine :size="14" />{{
            t('Alle Metadatengruppen', 'Every metadata group')
          }}</span
        ><span
          ><LockKeyhole :size="14" />{{
            t('Sichere Änderungen mit Backup', 'Safe changes with a backup')
          }}</span
        ><span
          ><Layers :size="14" />{{
            t('Eingebettete Daten & Sidecars', 'Embedded data & sidecars')
          }}</span
        >
      </div>
    </div>
    <template v-else>
      <div
        class="inspector-tabs"
        role="tablist"
        :aria-label="t('Inspector-Ansicht', 'Inspector view')"
      >
        <button
          v-for="tab in tabs"
          :key="tab.id"
          role="tab"
          :aria-selected="store.inspectorTab === tab.id"
          :class="{ active: store.inspectorTab === tab.id }"
          @click="store.inspectorTab = tab.id"
        >
          {{ tab.label
          }}<span
            v-if="tab.id === 'changes' && store.pendingCount"
            class="tiny-count"
            >{{ store.pendingCount }}</span
          >
        </button>
      </div>
      <div v-if="store.errors[store.selected[0] ?? '']" class="inline-error">
        <AlertTriangle :size="18" /><strong>{{
          t(
            'Metadaten konnten nicht gelesen werden',
            'Metadata could not be read',
          )
        }}</strong>
        <p>{{ store.errors[store.selected[0] ?? ''] }}</p>
      </div>
      <div v-else-if="!current" class="inspector-empty">
        <RefreshCw class="loading-spin" :size="22" />
        <p>{{ t('Metadaten werden gelesen …', 'Reading metadata …') }}</p>
      </div>
      <template v-else-if="store.inspectorTab === 'overview'">
        <div class="inspector-content">
          <div v-if="multi" class="multi-summary">
            <Layers :size="25" />
            <div>
              <strong
                >{{ store.selected.length }}
                {{ t('Dateien ausgewählt', 'files selected') }}</strong
              >
              <p>
                {{ store.selectedDocuments.length }}
                {{
                  t(
                    'analysiert · Werte nur dieser Teilmenge (max. 8)',
                    'analyzed · Values for this subset only (max. 8)',
                  )
                }}
              </p>
            </div>
            <button class="button small" @click="store.dialog = 'compare'">
              {{ t('Vergleichen', 'Compare') }}
            </button>
          </div>
          <template v-else
            ><div class="image-preview">
              <img
                v-if="store.previews.get(current.file.id)"
                :src="store.previews.get(current.file.id) ?? undefined"
                @error="store.previewFailed(current.file.id)"
                :alt="current.file.name"
              />
              <div v-else class="preview-unavailable">
                <ImageIcon :size="32" :stroke-width="1" /><span>{{
                  t('Keine Medienvorschau', 'No media preview')
                }}</span
                ><small>{{
                  t('Metadaten bleiben verfügbar', 'Metadata remains available')
                }}</small>
              </div>
              <span class="preview-badge">{{
                current.capabilities.format
              }}</span>
            </div>
            <div class="preview-caption">
              <strong :title="current.file.name">{{ current.file.name }}</strong
              ><span
                >{{ text(tagValue(current, ['ImageWidth'])) }} ×
                {{ text(tagValue(current, ['ImageHeight'])) }}<i>·</i
                >{{ sizeText(current.file.size) }}</span
              >
            </div></template
          >
          <div v-if="current.sidecar" class="sidecar-notice">
            <Layers :size="14" /><span
              >XMP {{ t('Sidecar verknüpft', 'sidecar linked') }}</span
            ><span v-if="conflicts.length" class="warning-text"
              >{{ conflicts.length }} {{ t('Konflikte', 'conflicts') }}</span
            >
          </div>
          <details v-if="conflicts.length" class="conflict-details">
            <summary>
              {{
                t(
                  'Abweichende Sidecar-Werte prüfen',
                  'Review conflicting sidecar values',
                )
              }}
            </summary>
            <div v-for="field in conflicts" :key="field.id">
              <strong>{{ de ? field.de : field.en }}</strong>
              <p v-for="source in ['embedded', 'sidecar']" :key="source">
                {{ source }}:
                {{
                  text(
                    current.tags.find(
                      (tag) =>
                        `${tag.location}:${tag.name}` === field.tag &&
                        tag.source === source,
                    )?.raw,
                  )
                }}
              </p>
            </div>
            <p>
              {{
                t(
                  'Bearbeitung verwendet Sidecar-Werte zuerst. Schreibziel im Editor ausdrücklich wählen.',
                  'Editing prefers sidecar values. Choose the write target explicitly in the editor.',
                )
              }}
            </p>
          </details>
          <section v-if="!multi" class="inspector-section">
            <header>
              <span>{{ t('AUFNAHME', 'CAPTURE') }}</span
              ><button class="text-button" @click="edit('dateTaken')">
                {{ t('Zeit ändern', 'Edit time') }}
              </button>
            </header>
            <div
              v-for="item in information"
              :key="item.label"
              class="capture-row"
            >
              <component :is="item.icon" :size="14" /><span>{{
                item.label
              }}</span
              ><strong>{{ text(item.value) || '—' }}</strong>
            </div>
            <div class="exposure-grid">
              <div v-for="item in exposure" :key="item.label">
                <strong>{{ item.value }}</strong
                ><span>{{ item.label }}</span>
              </div>
            </div>
          </section>
          <section class="inspector-section identity-section">
            <header>
              <span>{{ t('INHALT & RECHTE', 'CONTENT & RIGHTS') }}</span
              ><button
                class="text-button"
                :disabled="current.capabilities.write === 'readOnly'"
                @click="
                  store.contextId = '';
                  store.dialog = 'edit';
                "
              >
                {{ t('Bearbeiten', 'Edit') }}<Pencil :size="12" />
              </button>
            </header>
            <div
              v-for="field in identityFields"
              :key="field.id"
              class="metadata-field"
            >
              <label
                >{{ de ? field.de : field.en
                }}<code v-if="store.preferences.technical">{{
                  field.tag
                }}</code></label
              >
              <div
                class="field-display"
                :class="{
                  unset:
                    aggregate(store.selectedDocuments, field.id).state ===
                    'missing',
                  mixed: ['mixed', 'partial'].includes(
                    aggregate(store.selectedDocuments, field.id).state,
                  ),
                }"
              >
                <span
                  v-if="
                    field.id === 'keywords' &&
                    aggregate(store.selectedDocuments, field.id).state ===
                      'same'
                  "
                  class="keyword-list"
                  ><span
                    v-for="word in Array.isArray(
                      aggregate(store.selectedDocuments, field.id).value,
                    )
                      ? (aggregate(store.selectedDocuments, field.id)
                          .value as Json[])
                      : [aggregate(store.selectedDocuments, field.id).value]"
                    :key="text(word)"
                    class="keyword"
                    >{{ text(word) }}</span
                  ></span
                ><span v-else>{{ statusText(field.id) }}</span
                ><button
                  class="icon-button field-edit"
                  :disabled="
                    current.capabilities.write === 'readOnly' ||
                    current.file.readonly
                  "
                  :aria-label="`${de ? field.de : field.en} ${t('bearbeiten', 'edit')}`"
                  @click="edit(field.id)"
                >
                  <Pencil :size="13" />
                </button>
              </div>
            </div>
          </section>
          <section class="inspector-section">
            <header>
              <span>{{ t('STANDORT', 'LOCATION') }}</span
              ><button class="text-button" @click="edit('latitude')">
                {{ t('Bearbeiten', 'Edit') }}<Pencil :size="12" />
              </button>
            </header>
            <div class="location-display">
              <MapPin :size="17" />
              <div>
                <strong
                  >{{
                    text(tagValue(current, ['GPSLatitude'])) ||
                    t('Kein Standort gesetzt', 'No location set')
                  }}<template v-if="tagValue(current, ['GPSLongitude'])"
                    >, {{ text(tagValue(current, ['GPSLongitude'])) }}</template
                  ></strong
                ><span>{{
                  t(
                    'Keine Verbindung zu Kartendiensten',
                    'No connection to map services',
                  )
                }}</span>
              </div>
            </div>
            <button class="privacy-link" @click="store.dialog = 'privacy'">
              <LockKeyhole :size="13" />{{
                t('Datenschutz prüfen', 'Review privacy')
              }}<ArrowRight :size="13" />
            </button>
          </section>
          <div v-if="current.warnings.length" class="inspector-warnings">
            <p v-for="warning in current.warnings" :key="warning">
              <AlertTriangle :size="13" />{{ warning }}
            </p>
          </div>
          <div class="inspector-footnote">
            <Copyright :size="12" />{{
              t(
                'Datei ist Quelle · Cache wird überprüft',
                'File is the source · Cache is validated',
              )
            }}
          </div>
        </div>
      </template>
      <template v-else-if="store.inspectorTab === 'all'">
        <div class="tag-tools">
          <label class="search-field"
            ><Search :size="14" /><input
              v-model="tagSearch"
              :placeholder="
                t('Tags und Werte durchsuchen', 'Search tags and values')
              "
              :aria-label="
                t('Tags und Werte durchsuchen', 'Search tags and values')
              " /><button
              v-if="tagSearch"
              class="icon-button"
              :aria-label="t('Suche leeren', 'Clear search')"
              @click="tagSearch = ''"
            >
              <X :size="12" /></button
          ></label>
          <div class="tag-filters">
            <select
              v-model="group"
              :aria-label="t('Metadatengruppe', 'Metadata group')"
            >
              <option value="all">{{ t('Alle Gruppen', 'All groups') }}</option>
              <option v-for="g in groups" :key="g">{{ g }}</option></select
            ><select
              v-model="tagFilter"
              :aria-label="t('Tagfilter', 'Tag filter')"
            >
              <option value="all">{{ t('Alle Tags', 'All tags') }}</option>
              <option value="writable">
                {{ t('Bearbeitbar', 'Writable') }}
              </option>
              <option value="favorites">
                {{ t('Favoriten', 'Favorites') }}
              </option>
              <option value="changed">
                {{ t('Geändert', 'Changed') }}
              </option></select
            ><label class="checkbox-label"
              ><input v-model="raw" type="checkbox" />{{
                t('Roh', 'Raw')
              }}</label
            >
          </div>
          <div class="tag-count">
            <span>{{ tags.length }} {{ t('Tags', 'tags') }}</span
            ><button
              class="text-button"
              @click="
                copy(
                  tags
                    .map(
                      (tag) =>
                        `${tag.key}\t${text(raw ? tag.raw : tag.formatted)}`,
                    )
                    .join('\n'),
                  'view',
                )
              "
            >
              <Copy :size="12" />{{
                copied === 'view'
                  ? t('Kopiert', 'Copied')
                  : t('Ansicht kopieren', 'Copy view')
              }}
            </button>
          </div>
        </div>
        <div
          ref="tagScroll"
          class="tag-scroll"
          role="list"
          :aria-label="t('Metadaten', 'Metadata')"
        >
          <div
            :style="{
              height: `${virtualizer.getTotalSize()}px`,
              position: 'relative',
            }"
          >
            <template v-for="item in items" :key="String(item.key)"
              ><div
                v-if="tags[item.index]"
                class="tag-row"
                role="listitem"
                :style="{
                  transform: `translateY(${item.start}px)`,
                  height: `${item.size}px`,
                }"
              >
                <div class="tag-row-top">
                  <span>{{ tags[item.index]!.label }}</span
                  ><span class="tag-group"
                    >{{
                      tags[item.index]!.source === 'sidecar'
                        ? 'Sidecar · '
                        : ''
                    }}{{ tags[item.index]!.location }}</span
                  ><button
                    class="icon-button"
                    :class="{
                      starred: store.preferences.tagFavorites.includes(
                        `${tags[item.index]!.location}:${tags[item.index]!.name}`,
                      ),
                    }"
                    :aria-label="t('Tag favorisieren', 'Favorite tag')"
                    @click="favorite(tags[item.index]!)"
                  >
                    <Star :size="11" />
                  </button>
                </div>
                <code
                  v-if="store.preferences.technical"
                  class="tag-identity"
                  :title="tags[item.index]!.key"
                  >{{ tags[item.index]!.key }}</code
                >
                <div class="tag-value">
                  <span
                    :title="
                      text(
                        raw
                          ? tags[item.index]!.raw
                          : tags[item.index]!.formatted,
                      )
                    "
                    >{{
                      text(
                        raw
                          ? tags[item.index]!.raw
                          : tags[item.index]!.formatted,
                      ) || '—'
                    }}</span
                  ><span
                    v-if="!tags[item.index]!.writable"
                    class="tag-lock"
                    :title="
                      tags[item.index]!.derived
                        ? t(
                            'Abgeleiteter Wert. Wird aus anderen Tags berechnet.',
                            'Derived value. Calculated from other tags.',
                          )
                        : t(
                            'Kein freigegebener Schreibadapter für dieses Feld.',
                            'No supported write adapter for this field.',
                          )
                    "
                    ><LockKeyhole :size="11" /></span
                  ><button
                    class="icon-button"
                    :aria-label="t('Wert kopieren', 'Copy value')"
                    @click="
                      copy(
                        raw
                          ? tags[item.index]!.raw
                          : tags[item.index]!.formatted,
                        tags[item.index]!.id,
                      )
                    "
                  >
                    <Check
                      v-if="copied === tags[item.index]!.id"
                      :size="12"
                    /><Copy v-else :size="12" />
                  </button>
                </div></div
            ></template>
          </div>
        </div>
      </template>
      <div v-else class="inspector-content changes-panel">
        <div v-if="!store.pendingCount" class="inspector-empty">
          <Check :size="25" />
          <p>{{ t('Keine ausstehenden Änderungen', 'No pending changes') }}</p>
          <span>{{
            t(
              'Deine Originale sind unverändert.',
              'Your originals are unchanged.',
            )
          }}</span>
        </div>
        <template v-for="(draft, id) in store.drafts" :key="id"
          ><section class="draft-file">
            <strong>{{
              store.files.find((file) => file.id === id)?.name
            }}</strong>
            <div
              v-for="edit in draft.edits"
              :key="edit.field"
              class="draft-row"
            >
              <span>{{
                fields.find((f) => f.id === edit.field)?.[de ? 'de' : 'en']
              }}</span
              ><ArrowRight :size="12" /><span
                >{{ text(edit.value) || t('Entfernen', 'Remove')
                }}<small v-if="edit.mode !== 'replace'">
                  · {{ edit.mode }}</small
                ></span
              >
            </div>
            <p v-if="draft.privacy">
              {{ t('Datenschutz-Preset', 'Privacy preset') }}:
              {{ draft.privacy }}
            </p>
            <p v-if="draft.rename">→ {{ draft.rename }}</p>
            <button class="text-button" @click="delete store.drafts[id]">
              {{
                t(
                  'Ungespeicherte Änderungen verwerfen',
                  'Discard unsaved changes',
                )
              }}
            </button>
          </section></template
        >
        <p v-if="store.pendingCount" class="help-text">
          {{
            t(
              'Die genaue Vorher-nachher-Prüfung erfolgt vor dem Speichern.',
              'Exact before-and-after review happens before saving.',
            )
          }}
        </p>
      </div>
    </template>
  </aside>
</template>
