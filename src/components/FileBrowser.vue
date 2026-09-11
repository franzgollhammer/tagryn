<script setup lang="ts">
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue';
import { useVirtualizer } from '@tanstack/vue-virtual';
import {
  Image as ImageIcon,
  File,
  FolderOpen,
  Plus,
  ArrowDownWideNarrow,
  Columns3,
  Star,
  MoreHorizontal,
  SearchX,
} from '@lucide/vue';
import { useWorkspace } from '../stores/workspace';
import { useI18n } from '../i18n';
import { sizeText, tagValue, text } from '../domain/types';
import type { FileEntry } from '../domain/types';
import { Menu } from '@tauri-apps/api/menu';
import { native, review } from '../api';
const store = useWorkspace();
const { t } = useI18n();
const scroll = ref<HTMLElement>();
const width = ref(650);
const header = ref<HTMLElement>();
const tableColumns = computed(
  () =>
    `minmax(100px, 1fr) ${store.preferences.columns.map((column) => (column === 'rating' ? '30px' : width.value < 500 ? '52px' : '88px')).join(' ')} 22px`,
);
const tableMinWidth = computed(
  () =>
    160 +
    store.preferences.columns.reduce(
      (sum, column) =>
        sum + (column === 'rating' ? 38 : width.value < 500 ? 60 : 96),
      0,
    ),
);
const columns = computed(() =>
  store.grid ? Math.max(2, Math.floor(width.value / 176)) : 1,
);
const rowHeight = computed(() => (store.grid ? 180 : 49));
const virtualizer = useVirtualizer(
  computed(() => ({
    count: Math.ceil(store.visibleFiles.length / columns.value),
    getScrollElement: () => scroll.value ?? null,
    estimateSize: () => rowHeight.value,
    overscan: 5,
  })),
);
const rows = computed(() => virtualizer.value.getVirtualItems());
const rowFiles = (index: number): FileEntry[] =>
  store.visibleFiles.slice(index * columns.value, (index + 1) * columns.value);
const isSelected = (id: string) => store.selected.includes(id);
const caption = (file: FileEntry, column: string): string => {
  if (column === 'type') return file.extension.toUpperCase();
  if (column === 'size') return sizeText(file.size);
  if (column.startsWith('tag:')) {
    const selector = column.slice(4);
    const tags =
      store.documents
        .get(file.id)
        ?.tags.filter((tag) => `${tag.location}:${tag.name}` === selector) ??
      [];
    return tags.length
      ? tags.map((tag) => text(tag.formatted)).join(' · ')
      : store.summaries.get(file.id)?.[selector] || '—';
  }
  if (column === 'rating')
    return (
      text(tagValue(store.documents.get(file.id), ['Rating'])) ||
      store.summaries.get(file.id)?.Rating ||
      '—'
    );
  if (column === 'camera')
    return (
      text(tagValue(store.documents.get(file.id), ['Model'])) ||
      store.summaries.get(file.id)?.Model ||
      '—'
    );
  if (column === 'date')
    return (
      text(tagValue(store.documents.get(file.id), ['DateTimeOriginal'])).slice(
        0,
        10,
      ) ||
      store.summaries.get(file.id)?.DateTimeOriginal?.slice(0, 10) ||
      '—'
    );
  return file.modified.slice(0, 10);
};
const headings: Record<string, [string, string]> = {
  type: ['Typ', 'Type'],
  size: ['Größe', 'Size'],
  rating: ['★', '★'],
  camera: ['Kamera', 'Camera'],
  date: ['Aufnahme', 'Captured'],
  modified: ['Geändert', 'Modified'],
};
let observer: ResizeObserver | undefined;
let popupMenu: Menu | undefined;
onMounted(() => {
  observer = new ResizeObserver((entries) => {
    width.value = entries[0]?.contentRect.width ?? 650;
  });
  if (scroll.value) observer.observe(scroll.value);
});
watch(scroll, (element) => {
  observer?.disconnect();
  if (element) observer?.observe(element);
});
onBeforeUnmount(() => {
  observer?.disconnect();
  void popupMenu?.close();
});
let visibleGeneration = 0;
watch([rows, () => store.grid], async () => {
  const generation = ++visibleGeneration;
  const ids = rows.value.flatMap((row) => rowFiles(row.index).map((f) => f.id));
  for (let i = 0; i < Math.min(ids.length, 36); i += 2) {
    if (generation !== visibleGeneration) break;
    await Promise.all(
      ids.slice(i, i + 2).map(async (id) => {
        if (!review) await store.load(id);
        if (store.grid) await store.preview(id);
      }),
    );
  }
});
function key(event: KeyboardEvent) {
  const index = store.visibleFiles.findIndex(
    (f) => f.id === store.selected.at(-1),
  );
  let next = index;
  if (event.key === 'ArrowDown') next += columns.value;
  else if (event.key === 'ArrowUp') next -= columns.value;
  else if (event.key === 'ArrowRight' && store.grid) next += 1;
  else if (event.key === 'ArrowLeft' && store.grid) next -= 1;
  else if (event.key === 'Home') next = 0;
  else if (event.key === 'End') next = store.visibleFiles.length - 1;
  else if ((event.metaKey || event.ctrlKey) && event.key === 'a') {
    event.preventDefault();
    store.selected = store.visibleFiles.map((f) => f.id);
    void store.hydrateSelection();
    return;
  } else if (event.key === 'Enter') {
    store.dialog = 'edit';
    return;
  } else return;
  event.preventDefault();
  const file =
    store.visibleFiles[
      Math.max(0, Math.min(store.visibleFiles.length - 1, next))
    ];
  if (file) {
    store.select(file.id, event);
    virtualizer.value.scrollToIndex(Math.floor(next / columns.value));
  }
}
async function context(file: FileEntry) {
  if (!isSelected(file.id)) store.select(file.id);
  if (!native) {
    store.dialog = 'actions';
    return;
  }
  try {
    await popupMenu?.close();
    popupMenu = await Menu.new({
      items: [
        {
          id: 'edit',
          text: t('Metadaten bearbeiten', 'Edit metadata'),
          action: () => {
            store.dialog = 'edit';
          },
        },
        {
          id: 'compare',
          text: t('Vergleichen', 'Compare'),
          enabled: store.selected.length > 1,
          action: () => {
            store.dialog = 'compare';
          },
        },
        {
          id: 'privacy',
          text: t('Datenschutz prüfen', 'Review privacy'),
          action: () => {
            store.dialog = 'privacy';
          },
        },
        { item: 'Separator' },
        {
          id: 'copy',
          text: t('Dateipfad kopieren', 'Copy file path'),
          action: () => {
            void navigator.clipboard.writeText(file.path).catch(store.report);
          },
        },
        {
          id: 'refresh',
          text: t('Metadaten neu einlesen', 'Reload metadata'),
          action: () => {
            void store.load(file.id, true);
          },
        },
        {
          id: 'export',
          text: t('Exportieren …', 'Export …'),
          action: () => {
            store.dialog = 'export';
          },
        },
      ],
    });
    await popupMenu.popup();
  } catch (error) {
    store.report(error);
  }
}
</script>
<template>
  <section class="file-browser" aria-label="Dateien / Files">
    <div class="browser-subbar">
      <span
        >{{ store.visibleFiles.length.toLocaleString() }}
        {{ t('Dateien', 'files')
        }}<span v-if="store.selected.length" class="muted">
          · {{ store.selected.length }} {{ t('ausgewählt', 'selected') }}</span
        ></span
      >
      <div class="inline-actions">
        <label class="sort-control"
          ><ArrowDownWideNarrow :size="14" /><select
            v-model="store.sort"
            :aria-label="t('Sortierung', 'Sort')"
          >
            <option value="name">{{ t('Name', 'Name') }}</option>
            <option value="modified">{{ t('Geändert', 'Modified') }}</option>
            <option value="size">{{ t('Größe', 'Size') }}</option>
          </select></label
        ><button
          class="icon-button"
          :title="t('Spalten anpassen', 'Customize columns')"
          :aria-label="t('Spalten anpassen', 'Customize columns')"
          @click="store.dialog = 'columns'"
        >
          <Columns3 :size="16" />
        </button>
      </div>
    </div>
    <div v-if="!store.files.length" class="empty-workspace">
      <div class="empty-frame">
        <ImageIcon :size="40" :stroke-width="1.15" /><span
          class="detail-pixel"
        />
      </div>
      <h1>
        {{
          t('Jedes Bild hat mehr zu erzählen.', 'Every image has more to tell.')
        }}
      </h1>
      <p>
        {{
          t(
            'Dateien oder Ordner hier ablegen. Metadaten entdecken, vergleichen und sicher bearbeiten.',
            'Drop files or folders here. Explore, compare and safely edit their metadata.',
          )
        }}
      </p>
      <div class="empty-actions">
        <button class="button primary" @click="store.openFiles(true)">
          <FolderOpen :size="16" />{{
            t('Ordner öffnen', 'Open folder')
          }}</button
        ><button class="button" @click="store.openFiles()">
          <Plus :size="16" />{{ t('Dateien öffnen', 'Open files') }}
        </button>
      </div>
      <div class="empty-formats">
        JPEG · TIFF · HEIC · PNG · RAW <span>+ {{ t('weitere', 'more') }}</span>
      </div>
      <div class="empty-footnote">
        <span class="status-dot" />{{
          t(
            'Lokal auf deinem Gerät. Originale bleiben geschützt.',
            'Local to your device. Originals stay protected.',
          )
        }}
      </div>
    </div>
    <template v-else>
      <div
        v-if="!store.grid"
        ref="header"
        class="file-head"
        :style="{
          gridTemplateColumns: tableColumns,
        }"
      >
        <span>{{ t('Dateiname', 'Filename') }}</span
        ><span v-for="column in store.preferences.columns" :key="column">{{
          column.startsWith('tag:')
            ? column.slice(4)
            : t(...(headings[column] ?? ['', '']))
        }}</span
        ><span />
      </div>
      <div
        ref="scroll"
        class="files-scroll"
        tabindex="0"
        role="listbox"
        aria-multiselectable="true"
        :aria-label="
          t(
            'Dateien. Pfeiltasten zum Navigieren, Eingabe zum Bearbeiten.',
            'Files. Arrow keys to navigate, Enter to edit.',
          )
        "
        :aria-activedescendant="
          store.selected[0] ? `file-${store.selected[0]}` : undefined
        "
        @keydown="key"
        @scroll="
          header &&
          (header.scrollLeft = ($event.target as HTMLElement).scrollLeft)
        "
      >
        <div v-if="!store.visibleFiles.length" class="no-results">
          <SearchX :size="28" />
          <p>{{ t('Keine passenden Dateien', 'No matching files') }}</p>
          <button
            class="text-button"
            @click="
              store.search = '';
              store.filter = 'all';
              store.folder = '';
            "
          >
            {{ t('Filter zurücksetzen', 'Reset filters') }}
          </button>
        </div>
        <div
          :style="{
            height: `${virtualizer.getTotalSize()}px`,
            position: 'relative',
            width: '100%',
            minWidth: store.grid ? undefined : `${tableMinWidth}px`,
          }"
        >
          <div
            v-for="row in rows"
            :key="String(row.key)"
            class="virtual-file-row"
            :class="{ 'grid-row': store.grid }"
            :style="{
              transform: `translateY(${row.start}px)`,
              height: `${rowHeight}px`,
              gridTemplateColumns: store.grid
                ? `repeat(${columns}, 1fr)`
                : undefined,
            }"
          >
            <div
              v-for="file in rowFiles(row.index)"
              :id="`file-${file.id}`"
              :key="file.id"
              class="file-item"
              :class="{ selected: isSelected(file.id), thumbnail: store.grid }"
              role="option"
              :aria-selected="isSelected(file.id)"
              :aria-label="file.name"
              :style="
                !store.grid
                  ? {
                      gridTemplateColumns: tableColumns,
                    }
                  : undefined
              "
              @click="store.select(file.id, $event)"
              @dblclick="store.dialog = 'edit'"
              @contextmenu.prevent="context(file)"
            >
              <div v-if="store.grid" class="thumbnail-image">
                <img
                  v-if="store.previews.get(file.id)"
                  :src="store.previews.get(file.id) ?? undefined"
                  @error="store.previewFailed(file.id)"
                  alt=""
                  draggable="false"
                /><ImageIcon v-else :size="30" :stroke-width="1" /><span
                  class="format-chip"
                  >{{ file.extension.toUpperCase() }}</span
                ><span v-if="store.drafts[file.id]" class="pending-dot" />
              </div>
              <div class="file-name">
                <span
                  v-if="!store.grid"
                  class="file-glyph"
                  :class="{
                    raw: ['dng', 'nef', 'arw', 'cr3', 'raf'].includes(
                      file.extension,
                    ),
                  }"
                  ><File
                    v-if="['pdf', 'xmp'].includes(file.extension)"
                    :size="19"
                    :stroke-width="1.4" /><ImageIcon
                    v-else
                    :size="19"
                    :stroke-width="1.4"
                /></span>
                <div class="filename-text">
                  <span>{{ file.name }}</span
                  ><small v-if="store.grid">{{ sizeText(file.size) }}</small>
                </div>
                <span
                  v-if="store.drafts[file.id] && !store.grid"
                  class="pending-dot"
                /><span
                  v-if="store.errors[file.id]"
                  class="error-dot"
                  :title="store.errors[file.id]"
                  >!</span
                >
              </div>
              <template v-if="!store.grid"
                ><span
                  v-for="column in store.preferences.columns"
                  :key="column"
                  class="file-cell"
                  :class="{
                    mono: column === 'size',
                    rating: column === 'rating',
                  }"
                  ><Star
                    v-if="column === 'rating' && caption(file, column) !== '—'"
                    :size="10"
                  />{{ caption(file, column) }}</span
                ><button
                  class="row-more icon-button"
                  tabindex="-1"
                  :aria-label="t('Dateiaktionen', 'File actions')"
                  @click.stop="context(file)"
                >
                  <MoreHorizontal :size="16" /></button
              ></template>
            </div>
          </div>
        </div>
      </div>
    </template>
    <div v-if="store.files.length" class="browser-bottom">
      <span>{{
        t('⌘/Ctrl + Klick: Mehrfachauswahl', '⌘/Ctrl + click: select multiple')
      }}</span
      ><button class="text-button" @click="store.dialog = 'formats'">
        {{ t('Formatunterstützung', 'Format support') }} ↗
      </button>
    </div>
  </section>
</template>
