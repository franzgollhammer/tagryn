<script setup lang="ts">
import { onMounted, onBeforeUnmount, watchEffect, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { confirm } from '@tauri-apps/plugin-dialog';
import {
  Folder,
  FolderOpen,
  Plus,
  Search,
  LayoutGrid,
  List,
  SlidersHorizontal,
  ShieldCheck,
  PanelsTopLeft,
  Star,
  ChevronRight,
  Command,
  ArrowLeftRight,
  Pencil,
  X,
  CircleHelp,
  RefreshCw,
  Check,
  LockKeyhole,
  Ellipsis,
  Trash2,
  Layers,
  Download,
} from '@lucide/vue';
import FileBrowser from './components/FileBrowser.vue';
import Inspector from './components/Inspector.vue';
import PaneDivider from './components/PaneDivider.vue';
import WorkflowDialogs from './components/WorkflowDialogs.vue';
import { useWorkspace } from './stores/workspace';
import { useI18n } from './i18n';
import { api, native, review } from './api';
const store = useWorkspace();
const { t } = useI18n();
const searchInput = ref<HTMLInputElement>();
const systemTheme = window.matchMedia('(prefers-color-scheme: dark)');
const systemDark = ref(systemTheme.matches);
const themeChanged = (event: MediaQueryListEvent) => {
  systemDark.value = event.matches;
};
systemTheme.addEventListener('change', themeChanged);
watchEffect(() => {
  document.documentElement.dataset.theme =
    store.preferences.theme === 'system'
      ? systemDark.value
        ? 'dark'
        : 'light'
      : store.preferences.theme;
  document.documentElement.lang = store.preferences.language;
  document.documentElement.style.fontSize = `${store.preferences.fontSize}px`;
});
function folderName(path: string) {
  return path.split(/[\\/]/).at(-1) || path;
}
function favoriteFolder(path: string) {
  const index = store.preferences.favorites.indexOf(path);
  if (index >= 0) store.preferences.favorites.splice(index, 1);
  else store.preferences.favorites.push(path);
}
function key(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey)) return;
  const key = event.key.toLowerCase();
  if (key === 'k') {
    event.preventDefault();
    store.dialog = store.dialog === 'palette' ? '' : 'palette';
  } else if (key === 'o') {
    event.preventDefault();
    void store.openFiles(event.shiftKey);
  } else if (key === 's') {
    event.preventDefault();
    void store.reviewChanges();
  } else if (key === 'f') {
    event.preventDefault();
    searchInput.value?.focus();
  } else if (key === ',') {
    event.preventDefault();
    store.dialog = 'settings';
  } else if (key === 'e' && store.selected.length) {
    event.preventDefault();
    store.contextId = '';
    store.dialog = 'edit';
  } else if (key === 'd' && store.selected.length > 1) {
    event.preventDefault();
    store.dialog = 'compare';
  }
}
let unlistenClose: (() => void) | undefined;
onMounted(async () => {
  window.addEventListener('keydown', key);
  await store.init();
  if (native)
    requestAnimationFrame(() => {
      void api('frontend_ready').catch(store.report);
    });
  if (native)
    unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
      if (store.runningJobs.length) {
        event.preventDefault();
        store.notice = t(
          'Ein Job läuft. Zuerst abbrechen und laufende Datei abschließen lassen.',
          'A job is running. Cancel it and let the current file finish first.',
        );
        return;
      }
      if (store.pendingCount) {
        event.preventDefault();
        const discard = await confirm(
          t(
            'Ungespeicherte Änderungen verwerfen und Tagryn schließen?',
            'Discard unsaved changes and close Tagryn?',
          ),
          { title: 'Tagryn', kind: 'warning' },
        );
        if (discard) {
          store.drafts = {};
          await getCurrentWindow().close();
        }
      }
    });
});
onBeforeUnmount(() => {
  window.removeEventListener('keydown', key);
  systemTheme.removeEventListener('change', themeChanged);
  unlistenClose?.();
});
</script>
<template>
  <div class="app-shell" :class="{ 'native-app': native }">
    <header class="titlebar" data-tauri-drag-region>
      <div class="wordmark" data-tauri-drag-region>
        <img src="/brand/tagryn-small.svg" width="26" height="26" alt="" /><span
          data-tauri-drag-region
          >Tagryn</span
        ><span class="wordmark-caption" data-tauri-drag-region>{{
          t('Der Blick ins Detail', 'A closer look')
        }}</span>
      </div>
      <div class="titlebar-actions">
        <span v-if="store.engine" class="offline-indicator"
          ><span class="status-dot" />{{
            t('Lokal & privat', 'Local & private')
          }}</span
        ><button class="palette-trigger" @click="store.dialog = 'palette'">
          <Command :size="13" /><span>{{ t('Befehle', 'Commands') }}</span
          ><kbd>⌘ K</kbd>
        </button>
      </div>
    </header>
    <div v-if="!native" class="browser-notice">
      <span>{{
        review
          ? t(
              'Browservorschau · echte, lokal erzeugte Testdateien · Schreiben in der Desktop-App',
              'Browser review · real, locally generated test files · Writing in the desktop app',
            )
          : t(
              'Desktop-App starten: npm run desktop. Native Dateizugriffe sind im Browser nicht verfügbar.',
              'Start the desktop app: npm run desktop. Native file access is unavailable in the browser.',
            )
      }}</span>
    </div>
    <div v-if="store.engineError" class="engine-error" role="alert">
      <LockKeyhole :size="16" /><span>{{ store.engineError }}</span
      ><button class="text-button" @click="store.dialog = 'formats'">
        {{ t('Details', 'Details') }}
      </button>
    </div>
    <div
      class="workspace"
      :style="{
        gridTemplateColumns: `minmax(170px,${store.preferences.sidebar}px) 5px minmax(250px,1fr) 5px minmax(290px,${store.preferences.inspector}px)`,
      }"
    >
      <nav class="sidebar" :aria-label="t('Bibliothek', 'Library')">
        <div class="sidebar-top">
          <span class="eyebrow">{{ t('BIBLIOTHEK', 'LIBRARY') }}</span
          ><button
            class="icon-button"
            :aria-label="t('Ordner hinzufügen', 'Add folder')"
            @click="store.openFiles(true)"
          >
            <Plus :size="15" />
          </button>
        </div>
        <button
          class="nav-item"
          :class="{ active: !store.folder && store.filter === 'all' }"
          @click="
            store.folder = '';
            store.filter = 'all';
          "
        >
          <PanelsTopLeft :size="16" /><span>{{
            t('Alle Dateien', 'All files')
          }}</span
          ><span class="nav-count">{{ store.files.length }}</span></button
        ><button
          class="nav-item"
          :class="{ active: store.filter === 'changed' }"
          @click="
            store.folder = '';
            store.filter = 'changed';
          "
        >
          <Pencil :size="15" /><span>{{
            t('Ausstehende Änderungen', 'Pending changes')
          }}</span
          ><span v-if="store.pendingCount" class="nav-count pending">{{
            store.pendingCount
          }}</span>
        </button>
        <div class="sidebar-heading">
          <span>{{ t('ORDNER', 'FOLDERS') }}</span
          ><button
            class="icon-button"
            :aria-label="t('Ordner öffnen', 'Open folder')"
            @click="store.openFiles(true)"
          >
            <Plus :size="13" />
          </button>
        </div>
        <div class="sidebar-folders">
          <div
            v-for="path in store.folders"
            :key="path"
            class="folder-item"
            :class="{ active: store.folder === path }"
          >
            <button
              class="nav-item"
              :title="path"
              @click="
                store.folder = path;
                store.filter = 'all';
              "
            >
              <FolderOpen v-if="store.folder === path" :size="15" /><Folder
                v-else
                :size="15"
              /><span>{{ folderName(path) }}</span
              ><span class="nav-count">{{
                store.files.filter((f) => f.folder === path).length
              }}</span></button
            ><button
              class="folder-favorite icon-button"
              :class="{ starred: store.preferences.favorites.includes(path) }"
              :aria-label="t('Ordner favorisieren', 'Favorite folder')"
              @click="favoriteFolder(path)"
            >
              <Star :size="12" />
            </button>
          </div>
          <button
            v-if="!store.folders.length"
            class="add-folder"
            @click="store.openFiles(true)"
          >
            <Folder :size="14" />{{ t('Ordner hinzufügen', 'Add a folder') }}
          </button>
        </div>
        <div class="sidebar-heading">
          <span>{{ t('FAVORITEN', 'FAVORITES') }}</span
          ><Star :size="11" />
        </div>
        <div v-if="!store.preferences.favorites.length" class="sidebar-hint">
          {{
            t(
              'Häufig verwendete Ordner anheften.',
              'Pin your most-used folders.',
            )
          }}
        </div>
        <button
          v-for="path in store.preferences.favorites"
          :key="path"
          class="nav-item"
          :title="path"
          @click="
            store.folder = path;
            store.scan([path]);
          "
        >
          <Star :size="14" /><span>{{ folderName(path) }}</span>
        </button>
        <div class="sidebar-heading">
          <span>{{ t('GESPEICHERTE ANSICHTEN', 'SAVED VIEWS') }}</span
          ><button
            class="icon-button"
            :aria-label="t('Ansicht speichern', 'Save view')"
            @click="store.dialog = 'columns'"
          >
            <Plus :size="13" />
          </button>
        </div>
        <button
          class="nav-item"
          :class="{ active: store.filter === 'raw' }"
          @click="
            store.folder = '';
            store.filter = 'raw';
          "
        >
          <Layers :size="14" /><span>{{ t('RAW-Dateien', 'RAW files') }}</span>
        </button>
        <div
          v-for="(view, index) in store.preferences.views"
          :key="index"
          class="saved-view"
        >
          <button
            class="nav-item"
            @click="
              store.search = view.search;
              store.filter = view.filter;
              store.sort = view.sort;
              store.preferences.columns = [...view.columns];
            "
          >
            <SlidersHorizontal :size="14" /><span>{{ view.name }}</span></button
          ><button
            class="icon-button"
            :aria-label="t('Ansicht löschen', 'Delete view')"
            @click="store.preferences.views.splice(index, 1)"
          >
            <Trash2 :size="12" />
          </button>
        </div>
        <div class="sidebar-bottom">
          <div class="sidebar-heading">
            <span>{{ t('WERKZEUGE', 'TOOLS') }}</span>
          </div>
          <button
            class="nav-item"
            :disabled="!store.selected.length"
            @click="store.dialog = 'privacy'"
          >
            <ShieldCheck :size="16" /><span>{{
              t('Datenschutz', 'Privacy')
            }}</span></button
          ><button class="nav-item" @click="store.dialog = 'presets'">
            <SlidersHorizontal :size="15" /><span>{{
              t('Vorlagen', 'Presets')
            }}</span></button
          ><button class="nav-item" @click="store.dialog = 'history'">
            <RefreshCw :size="15" /><span>{{
              t('Jobs & Wiederherstellung', 'Jobs & recovery')
            }}</span
            ><span v-if="store.runningJobs.length" class="nav-count">{{
              store.runningJobs.length
            }}</span>
          </button>
          <div class="sidebar-settings">
            <button class="text-button" @click="store.dialog = 'settings'">
              <SlidersHorizontal :size="14" />{{
                t('Einstellungen', 'Settings')
              }}</button
            ><button
              class="icon-button"
              :aria-label="
                t('Hilfe und Formatgrenzen', 'Help and format limits')
              "
              @click="store.dialog = 'formats'"
            >
              <CircleHelp :size="15" />
            </button>
          </div>
        </div>
      </nav>
      <PaneDivider
        :value="store.preferences.sidebar"
        :min="170"
        :max="300"
        :label="t('Breite der Seitenleiste', 'Sidebar width')"
        @resize="(value) => (store.preferences.sidebar = value)"
      />
      <main class="main-pane">
        <header class="workspace-toolbar">
          <div class="breadcrumb">
            <Folder :size="16" /><span>{{
              store.folder
                ? folderName(store.folder)
                : t('Alle Dateien', 'All files')
            }}</span
            ><ChevronRight :size="13" /><span class="breadcrumb-muted">{{
              t('Metadaten', 'Metadata')
            }}</span>
          </div>
          <button class="button small" @click="store.openFiles()">
            <Plus :size="14" />{{ t('Öffnen', 'Open') }}
          </button>
        </header>
        <div class="file-toolbar">
          <label class="search-field file-search"
            ><Search :size="15" /><input
              ref="searchInput"
              v-model="store.search"
              :placeholder="
                t(
                  'Dateien, Tags oder Werte suchen …',
                  'Search files, tags or values …',
                )
              "
              :aria-label="
                t(
                  'Dateien, Tags oder Werte suchen',
                  'Search files, tags or values',
                )
              "
            /><button
              v-if="store.search"
              class="icon-button"
              :aria-label="t('Suche leeren', 'Clear search')"
              @click="store.search = ''"
            >
              <X :size="13" /></button
            ><kbd v-else>⌘ F</kbd></label
          >
          <div class="segmented">
            <button
              :class="{ active: !store.grid }"
              :aria-pressed="!store.grid"
              :aria-label="t('Listenansicht', 'List view')"
              @click="store.grid = false"
            >
              <List :size="16" /></button
            ><button
              :class="{ active: store.grid }"
              :aria-pressed="store.grid"
              :aria-label="t('Thumbnail-Raster', 'Thumbnail grid')"
              @click="store.grid = true"
            >
              <LayoutGrid :size="15" />
            </button>
          </div>
        </div>
        <div v-if="store.search" class="search-hint">
          <span
            >{{
              t(
                'Metadatensuche: bisher eingelesene Dateien',
                'Metadata search: files read so far',
              )
            }}
            ({{
              Math.max(
                store.indexedCount,
                store.summaries.size,
                store.documents.size,
              )
            }})</span
          ><button class="text-button" @click="store.indexAll">
            {{ t('Alle einlesen', 'Read all') }}
          </button>
        </div>
        <div class="selection-toolbar">
          <button
            class="tool-button"
            :disabled="!store.selected.length"
            @click="
              store.contextId = '';
              store.dialog = 'edit';
            "
          >
            <Pencil :size="14" />{{ t('Bearbeiten', 'Edit') }}</button
          ><button
            class="tool-button"
            :disabled="store.selected.length < 2"
            @click="store.dialog = 'compare'"
          >
            <ArrowLeftRight :size="15" />{{
              t('Vergleichen', 'Compare')
            }}</button
          ><button
            class="tool-button"
            :disabled="!store.selected.length"
            @click="store.dialog = 'export'"
          >
            <Download :size="14" />{{ t('Exportieren', 'Export') }}
          </button>
          <div class="toolbar-spacer" />
          <button
            class="icon-button"
            :aria-label="t('Weitere Aktionen', 'More actions')"
            @click="store.dialog = 'actions'"
          >
            <Ellipsis :size="19" />
          </button>
        </div>
        <FileBrowser />
      </main>
      <PaneDivider
        :value="store.preferences.inspector"
        :min="300"
        :max="540"
        reverse
        :label="t('Breite des Inspectors', 'Inspector width')"
        @resize="(value) => (store.preferences.inspector = value)"
      />
      <Inspector />
    </div>
    <div
      v-if="store.pendingCount"
      class="pending-tray"
      role="region"
      :aria-label="t('Ausstehende Änderungen', 'Pending changes')"
    >
      <div class="pending-description">
        <span class="pending-dot" /><strong
          >{{ store.pendingCount }}
          {{ t('Dateien mit Änderungen', 'files with changes') }}</strong
        ><span>{{ t('Noch nicht gespeichert', 'Not saved yet') }}</span>
      </div>
      <button class="text-button" @click="store.dialog = 'discard'">
        {{ t('Verwerfen', 'Discard') }}</button
      ><button
        class="button"
        :disabled="!!store.busy"
        @click="store.reviewChanges"
      >
        {{ t('Änderungen prüfen', 'Review changes') }}</button
      ><button
        class="button primary"
        :disabled="!!store.busy || !native"
        @click="store.reviewChanges"
      >
        <ShieldCheck :size="15" />{{ t('Speichern', 'Save') }}
      </button>
    </div>
    <div
      v-if="store.runningJobs.length || store.busy"
      class="job-tray"
      aria-live="polite"
    >
      <template v-if="store.busy"
        ><RefreshCw :size="14" class="loading-spin" /><span>{{
          store.busy
        }}</span
        ><button
          v-if="store.busy.includes('Suche') || store.busy.includes('Indexing')"
          class="text-button"
          @click="store.busy = ''"
        >
          {{ t('Stoppen', 'Stop') }}
        </button></template
      ><template v-for="job in store.runningJobs" :key="job.id"
        ><RefreshCw :size="14" class="loading-spin" /><button
          class="text-button"
          @click="store.dialog = 'history'"
        >
          {{
            job.kind === 'scan'
              ? t('Dateien einlesen', 'Reading files')
              : t('Änderungen speichern', 'Saving changes')
          }}
          · {{ job.completed }}/{{ job.total
          }}<span v-if="job.errors">
            · {{ job.errors }} {{ t('Fehler', 'errors') }}</span
          ></button
        ><progress
          :value="job.completed"
          :max="Math.max(job.total, 1)"
          :aria-label="t('Jobfortschritt', 'Job progress')"
        /><button
          class="text-button"
          @click="api('cancel_job', { jobId: job.id }).catch(store.report)"
        >
          {{ t('Abbrechen', 'Cancel') }}
        </button></template
      >
    </div>
    <footer class="statusbar">
      <span
        ><span class="status-dot" />{{
          store.engine
            ? `ExifTool ${store.engine}`
            : t('Engine wird verbunden …', 'Connecting engine …')
        }}</span
      ><span><LockKeyhole :size="11" />{{ t('Offline', 'Offline') }}</span>
      <div class="toolbar-spacer" />
      <span v-if="store.pendingCount">{{
        t('Ungespeicherte Änderungen', 'Unsaved changes')
      }}</span
      ><span v-else
        ><Check :size="12" />{{
          t('Keine ausstehenden Änderungen', 'No pending changes')
        }}</span
      >
    </footer>
    <div v-if="store.notice" class="toast" role="alert">
      <span>{{ store.notice }}</span
      ><button
        class="icon-button"
        :aria-label="t('Hinweis schließen', 'Dismiss notification')"
        @click="store.notice = ''"
      >
        <X :size="16" />
      </button>
    </div>
    <WorkflowDialogs />
  </div>
</template>
