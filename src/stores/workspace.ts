import { computed, ref, shallowRef, watch } from 'vue';
import { defineStore } from 'pinia';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { open, save } from '@tauri-apps/plugin-dialog';
import { api, native, review, reviewDocuments } from '../api';
import type {
  ChangeRequest,
  Edit,
  FileEntry,
  Job,
  Json,
  MetadataDocument,
  Plan,
  Preferences,
} from '../domain/types';

const active = (j: Job) =>
  ['running', 'queued', 'cancelling'].includes(j.status);
export const useWorkspace = defineStore('workspace', () => {
  const files = shallowRef<FileEntry[]>([]);
  const documents = shallowRef(new Map<string, MetadataDocument>());
  const previews = shallowRef(new Map<string, string | null>());
  const selected = ref<string[]>([]);
  const folder = ref('');
  const search = ref('');
  const filter = ref('all');
  const sort = ref('name');
  const grid = ref(false);
  const inspectorTab = ref<'overview' | 'all' | 'changes'>('overview');
  const drafts = ref<Record<string, ChangeRequest>>({});
  const jobs = shallowRef<Job[]>([]);
  const busy = ref('');
  const errors = ref<Record<string, string>>({});
  const notice = ref('');
  const engine = ref('');
  const engineError = ref('');
  const plan = shallowRef<Plan | null>(null);
  const dialog = ref('');
  const contextId = ref('');
  const indexedCount = ref(0);
  const searchMatches = shallowRef(new Set<string>());
  const resultStates = new Map<string, string>();
  const submitted = new Map<string, Record<string, string>>();
  const preferences = ref<Preferences>({
    theme: 'system',
    language: 'de',
    fontSize: 13,
    technical: false,
    recursive: true,
    sidebar: 216,
    inspector: 370,
    favorites: [],
    tagFavorites: [],
    columns: ['type', 'size', 'rating'],
    views: [],
    presets: [],
  });
  const loading = new Map<string, Promise<void>>();
  const previewLoading = new Set<string>();
  const documentSizes = new Map<string, number>();
  const summaries = shallowRef(new Map<string, Record<string, string>>());
  const fileById = computed(
    () => new Map(files.value.map((file) => [file.id, file])),
  );
  const selection = computed(() =>
    selected.value
      .map((id) => fileById.value.get(id))
      .filter((v): v is FileEntry => !!v),
  );
  const selectedDocuments = computed(() =>
    selected.value
      .slice(0, 8)
      .map((id) => documents.value.get(id))
      .filter((v): v is MetadataDocument => !!v),
  );
  const current = computed(() => documents.value.get(selected.value[0] ?? ''));
  const folders = computed(() => [
    ...new Set(files.value.map((f) => f.folder)),
  ]);
  const pendingCount = computed(() => Object.keys(drafts.value).length);
  const runningJobs = computed(() => jobs.value.filter(active));
  const visibleFiles = computed(() => {
    const query = search.value.toLocaleLowerCase();
    return files.value
      .filter((file) => {
        if (
          folder.value &&
          file.folder !== folder.value &&
          !file.folder.startsWith(`${folder.value}/`) &&
          !file.folder.startsWith(`${folder.value}\\`)
        )
          return false;
        if (filter.value === 'changed' && !drafts.value[file.id]) return false;
        if (
          filter.value === 'raw' &&
          ![
            'dng',
            'cr2',
            'cr3',
            'nef',
            'arw',
            'raf',
            'orf',
            'rw2',
            'pef',
          ].includes(file.extension)
        )
          return false;
        if (
          filter.value === 'gps' &&
          !summaries.value.get(file.id)?.GPSLatitude &&
          !documents.value
            .get(file.id)
            ?.tags.some((tag) => tag.name === 'GPSLatitude')
        )
          return false;
        if (!query) return true;
        if (
          `${file.name} ${file.extension}`.toLocaleLowerCase().includes(query)
        )
          return true;
        return (
          searchMatches.value.has(file.id) ||
          (documents.value
            .get(file.id)
            ?.tags.some((t) =>
              `${t.key} ${JSON.stringify(t.raw)} ${JSON.stringify(t.formatted)}`
                .toLocaleLowerCase()
                .includes(query),
            ) ??
            false)
        );
      })
      .sort((a, b) =>
        sort.value === 'size'
          ? b.size - a.size
          : sort.value === 'modified'
            ? b.modified.localeCompare(a.modified)
            : a.name.localeCompare(b.name, undefined, { numeric: true }),
      );
  });
  function report(error: unknown) {
    notice.value = error instanceof Error ? error.message : String(error);
  }
  function mergeFiles(batch: FileEntry[]) {
    const map = new Map(files.value.map((f) => [f.id, f]));
    for (const file of batch) map.set(file.id, file);
    files.value = [...map.values()];
    if (!selected.value.length && batch[0]) select(batch[0].id);
  }
  async function load(id: string, refresh = false): Promise<void> {
    if (!refresh && documents.value.has(id)) return;
    if (loading.has(id)) return loading.get(id);
    const promise = (async () => {
      try {
        const document = await api<MetadataDocument>('read_metadata', {
          fileId: id,
          refresh,
        });
        const map = new Map(documents.value);
        map.set(id, document);
        documentSizes.set(id, JSON.stringify(document).length * 2);
        let bytes = [...map.keys()].reduce(
          (sum, key) => sum + (documentSizes.get(key) ?? 0),
          0,
        );
        const pinned = new Set(selected.value.slice(0, 8));
        for (const key of map.keys()) {
          if (map.size <= 128 && bytes <= 64 * 1024 * 1024) break;
          if (key !== id && !pinned.has(key)) {
            map.delete(key);
            bytes -= documentSizes.get(key) ?? 0;
            documentSizes.delete(key);
          }
        }
        const summary: Record<string, string> = {};
        for (const tag of document.tags)
          if (
            ['Rating', 'Model', 'DateTimeOriginal', 'GPSLatitude'].includes(
              tag.name,
            ) ||
            preferences.value.columns.includes(
              `tag:${tag.location}:${tag.name}`,
            )
          )
            summary[
              ['Rating', 'Model', 'DateTimeOriginal', 'GPSLatitude'].includes(
                tag.name,
              )
                ? tag.name
                : `${tag.location}:${tag.name}`
            ] =
              typeof tag.formatted === 'string'
                ? tag.formatted
                : JSON.stringify(tag.formatted);
        const compact = new Map(summaries.value);
        compact.set(id, summary);
        summaries.value = compact;
        documents.value = map;
        delete errors.value[id];
      } catch (e) {
        errors.value[id] = String(e);
      } finally {
        loading.delete(id);
      }
    })();
    loading.set(id, promise);
    return promise;
  }
  async function preview(id: string) {
    if (previews.value.has(id) || previewLoading.has(id)) return;
    previewLoading.add(id);
    try {
      const value = review
        ? `/review-images/${encodeURIComponent(fileById.value.get(id)?.name ?? '')}`
        : await api<string | null>('read_preview', { fileId: id });
      const map = new Map(previews.value);
      map.set(id, value);
      if (map.size > 80) {
        const key = map.keys().next().value;
        if (key) map.delete(key);
      }
      previews.value = map;
    } catch {
      const map = new Map(previews.value);
      map.set(id, null);
      previews.value = map;
    } finally {
      previewLoading.delete(id);
    }
  }
  function previewFailed(id: string) {
    const map = new Map(previews.value);
    map.set(id, null);
    previews.value = map;
  }
  function select(id: string, event?: MouseEvent | KeyboardEvent) {
    if (event?.shiftKey && selected.value.length) {
      const a = visibleFiles.value.findIndex((f) => f.id === selected.value[0]);
      const b = visibleFiles.value.findIndex((f) => f.id === id);
      selected.value = visibleFiles.value
        .slice(Math.min(a, b), Math.max(a, b) + 1)
        .map((f) => f.id);
    } else if (event?.metaKey || event?.ctrlKey)
      selected.value = selected.value.includes(id)
        ? selected.value.filter((v) => v !== id)
        : [...selected.value, id];
    else selected.value = [id];
    void hydrateSelection();
  }
  async function hydrateSelection() {
    const ids = selected.value.slice(0, 8);
    for (let i = 0; i < ids.length; i += 2)
      await Promise.all(ids.slice(i, i + 2).map((id) => load(id)));
    if (ids[0]) void preview(ids[0]);
  }
  async function scan(paths: string[]) {
    try {
      await api('open_paths', {
        paths,
        recursive: preferences.value.recursive,
      });
    } catch (e) {
      report(e);
    }
  }
  async function openFiles(directory = false) {
    try {
      const chosen = await open({
        directory,
        multiple: true,
        title: directory
          ? 'Tagryn — Ordner öffnen / Open folder'
          : 'Tagryn — Dateien öffnen / Open files',
      });
      if (chosen) await scan(Array.isArray(chosen) ? chosen : [chosen]);
    } catch (e) {
      report(e);
    }
  }
  async function draft(
    edits: Edit[],
    ids = [...selected.value],
    extra: Partial<ChangeRequest> = {},
  ) {
    busy.value =
      preferences.value.language === 'de'
        ? 'Metadaten vorbereiten …'
        : 'Preparing metadata …';
    try {
      for (const id of ids) {
        await load(id);
        const doc = documents.value.get(id);
        if (!doc) throw new Error(errors.value[id] ?? 'Metadata unavailable');
        const previous = drafts.value[id];
        const merged = new Map(previous?.edits.map((e) => [e.field, e]) ?? []);
        for (const edit of edits) merged.set(edit.field, edit);
        drafts.value[id] = {
          fileId: id,
          expected: previous?.expected ?? doc.fingerprint,
          sidecarExpected: previous?.sidecarExpected ?? doc.sidecarFingerprint,
          edits: [...merged.values()],
          privacy: previous?.privacy ?? null,
          syncLegacy: previous?.syncLegacy ?? false,
          forceSidecar: previous?.forceSidecar ?? false,
          rename: previous?.rename ?? null,
          ...extra,
        };
      }
      plan.value = null;
    } catch (e) {
      report(e);
    } finally {
      busy.value = '';
    }
  }
  async function reviewChanges() {
    if (!pendingCount.value) return;
    busy.value =
      preferences.value.language === 'de'
        ? 'Dateien prüfen und Änderungen planen …'
        : 'Checking files and planning changes …';
    try {
      plan.value = await api<Plan>('plan_changes', {
        requests: Object.values(drafts.value),
      });
      dialog.value = 'review';
    } catch (e) {
      report(e);
    } finally {
      busy.value = '';
    }
  }
  async function execute() {
    if (!plan.value) return;
    const snapshot = Object.fromEntries(
      Object.entries(drafts.value).map(([id, draft]) => [
        id,
        JSON.stringify(draft),
      ]),
    );
    try {
      const id = await api<string>('execute_plan', { planId: plan.value.id });
      submitted.set(id, snapshot);
      dialog.value = '';
      plan.value = null;
    } catch (e) {
      report(e);
    }
  }
  async function indexAll() {
    busy.value =
      preferences.value.language === 'de'
        ? 'Metadaten für Suche einlesen …'
        : 'Indexing metadata for search …';
    const ids = files.value.map((f) => f.id);
    try {
      for (let i = 0; i < ids.length; i += 2) {
        if (!busy.value) break;
        await Promise.all(ids.slice(i, i + 2).map((id) => load(id)));
      }
    } finally {
      busy.value = '';
      await updateSearch();
    }
  }
  let searchGeneration = 0;
  async function updateSearch() {
    if (!native) return;
    const generation = ++searchGeneration;
    try {
      const result = await api<{ ids: string[]; indexed: number }>(
        'search_metadata',
        { query: search.value },
      );
      if (generation === searchGeneration) {
        searchMatches.value = new Set(result.ids);
        indexedCount.value = result.indexed;
      }
    } catch (e) {
      report(e);
    }
  }
  let searchTimer: ReturnType<typeof setTimeout>;
  watch(search, () => {
    searchMatches.value = new Set();
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      void updateSearch();
    }, 180);
  });
  async function exportFiles(format: 'csv' | 'json') {
    try {
      const path = await save({
        defaultPath: `tagryn-metadata.${format}`,
        filters: [{ name: format.toUpperCase(), extensions: [format] }],
      });
      if (!path) return;
      busy.value = 'Export …';
      await api('export_metadata', {
        fileIds: [...selected.value],
        path,
        format,
      });
      notice.value = `${format.toUpperCase()} → ${path}`;
    } catch (e) {
      report(e);
    } finally {
      busy.value = '';
    }
  }
  async function init() {
    if (review) {
      try {
        const session = await reviewDocuments();
        const docs = session.files;
        documents.value = new Map(docs.map((d) => [d.file.id, d]));
        files.value = session.entries ?? docs.map((d) => d.file);
        engine.value = '13.59 · Review';
        if (docs[0]) {
          selected.value = [docs[0].file.id];
          void preview(docs[0].file.id);
        }
      } catch (e) {
        report(e);
      }
      return;
    }
    if (!native) return;
    await listen<string>('tagryn:menu', (event) => {
      const action = event.payload;
      if (action === 'open' || action === 'folder')
        void openFiles(action === 'folder');
      else if (action === 'save') void reviewChanges();
      else if (action === 'quit') void getCurrentWebviewWindow().close();
      else if (action !== 'compare' || selected.value.length > 1)
        dialog.value = action;
    });
    await listen<FileEntry[]>('tagryn:files', (event) =>
      mergeFiles(event.payload),
    );
    await listen<Job>('tagryn:job', (event) => {
      const value = event.payload;
      jobs.value = [
        value,
        ...jobs.value.filter((j) => j.id !== value.id),
      ].slice(0, 100);
      if (value.kind === 'write')
        for (const result of value.results)
          if (['success', 'warning', 'restored'].includes(result.status)) {
            const key = `${value.id}:${result.fileId}`;
            if (resultStates.get(key) === result.status) continue;
            resultStates.set(key, result.status);
            if (
              JSON.stringify(drafts.value[result.fileId]) ===
              submitted.get(value.id)?.[result.fileId]
            )
              delete drafts.value[result.fileId];
            const map = new Map(documents.value);
            map.delete(result.fileId);
            documents.value = map;
            if (selected.value.includes(result.fileId))
              void load(result.fileId, true);
          }
    });
    await listen<string[]>('tagryn:open', (event) => {
      void scan(event.payload);
    });
    await getCurrentWebviewWindow().onDragDropEvent((event) => {
      if (event.payload.type === 'drop') void scan(event.payload.paths);
    });
    try {
      const settings = await api<Partial<Preferences> | null>('get_setting', {
        key: 'preferences',
      });
      if (settings) preferences.value = { ...preferences.value, ...settings };
      jobs.value = await api<Job[]>('job_history');
      const status = await api<{ version: string }>('engine_status');
      engine.value = status.version;
      const paths = await api<string[]>('initial_paths');
      if (paths.length) await scan(paths);
    } catch (e) {
      engineError.value = String(e);
    }
  }
  let preferenceTimer: ReturnType<typeof setTimeout>;
  watch(
    preferences,
    (value) => {
      clearTimeout(preferenceTimer);
      if (native)
        preferenceTimer = setTimeout(() => {
          void api('set_setting', {
            key: 'preferences',
            value: value as unknown as Json,
          }).catch(report);
        }, 250);
    },
    { deep: true },
  );
  return {
    files,
    fileById,
    documents,
    summaries,
    previews,
    selected,
    selection,
    selectedDocuments,
    current,
    folders,
    folder,
    search,
    filter,
    sort,
    grid,
    visibleFiles,
    inspectorTab,
    drafts,
    jobs,
    runningJobs,
    pendingCount,
    busy,
    errors,
    notice,
    engine,
    engineError,
    plan,
    dialog,
    contextId,
    preferences,
    indexedCount,
    report,
    mergeFiles,
    select,
    load,
    preview,
    previewFailed,
    scan,
    openFiles,
    draft,
    reviewChanges,
    execute,
    indexAll,
    exportFiles,
    init,
    hydrateSelection,
  };
});
