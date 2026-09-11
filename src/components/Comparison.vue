<script setup lang="ts">
import { computed, ref } from 'vue';
import { useVirtualizer } from '@tanstack/vue-virtual';
import { Copy, Search } from '@lucide/vue';
import { useWorkspace } from '../stores/workspace';
import { useI18n } from '../i18n';
import { text } from '../domain/types';
const store = useWorkspace();
const { t } = useI18n();
const differences = ref(true);
const query = ref('');
const scroll = ref<HTMLElement>();
const docs = computed(() => store.selectedDocuments.slice(0, 8));
const rows = computed(() => {
  const keys = [
    ...new Set(docs.value.flatMap((doc) => doc.tags.map((tag) => tag.id))),
  ];
  return keys
    .map((key) => {
      const tags = docs.value.map((doc) =>
        doc.tags.find((tag) => tag.id === key),
      );
      const values = tags.map((tag) => (tag ? text(tag.raw) : null));
      const same = values.every((value) => value === values[0]);
      return { key, label: tags.find(Boolean)?.label ?? key, values, same };
    })
    .filter(
      (row) =>
        (!differences.value || !row.same) &&
        `${row.key} ${row.values.join(' ')}`
          .toLowerCase()
          .includes(query.value.toLowerCase()),
    );
});
const virtualizer = useVirtualizer(
  computed(() => ({
    count: rows.value.length,
    getScrollElement: () => scroll.value ?? null,
    estimateSize: () => 57,
    overscan: 8,
  })),
);
const template = computed(
  () => `220px repeat(${docs.value.length}, minmax(180px,1fr))`,
);
async function copy() {
  try {
    await navigator.clipboard.writeText(
      [
        'Tag\t' + docs.value.map((doc) => doc.file.name).join('\t'),
        ...rows.value.map(
          (row) => row.key + '\t' + row.values.map((v) => v ?? '∅').join('\t'),
        ),
      ].join('\n'),
    );
    store.notice = t('Vergleich kopiert', 'Comparison copied');
  } catch (e) {
    store.report(e);
  }
}
</script>
<template>
  <div class="comparison-toolbar">
    <label class="search-field"
      ><Search :size="15" /><input
        v-model="query"
        :placeholder="t('Im Vergleich suchen', 'Search comparison')"
        :aria-label="t('Im Vergleich suchen', 'Search comparison')" /></label
    ><label class="checkbox-label"
      ><input v-model="differences" type="checkbox" />{{
        t('Nur Unterschiede', 'Differences only')
      }}</label
    ><button class="button small" @click="copy">
      <Copy :size="13" />{{ t('Kopieren', 'Copy') }}
    </button>
  </div>
  <p v-if="store.selected.length > 8" class="help-text">
    {{
      t(
        'Vergleich zeigt die ersten acht ausgewählten Dateien.',
        'Comparison displays the first eight selected files.',
      )
    }}
  </p>
  <p class="help-text">
    {{ rows.length }}
    {{
      t(
        'Tags · ∅ = fehlend · Gruppe, Instanz und Herkunft bleiben getrennt.',
        'tags · ∅ = missing · Group, instance and source remain distinct.',
      )
    }}
  </p>
  <div
    ref="scroll"
    class="comparison-scroll"
    role="table"
    :aria-label="t('Metadatenvergleich', 'Metadata comparison')"
  >
    <div
      class="comparison-head"
      role="row"
      :style="{ gridTemplateColumns: template }"
    >
      <strong role="columnheader">{{ t('Metadaten', 'Metadata') }}</strong
      ><strong v-for="doc in docs" :key="doc.file.id" role="columnheader">{{
        doc.file.name
      }}</strong>
    </div>
    <div
      :style="{
        height: `${virtualizer.getTotalSize()}px`,
        position: 'relative',
        minWidth: `${220 + docs.length * 180}px`,
      }"
    >
      <template
        v-for="item in virtualizer.getVirtualItems()"
        :key="String(item.key)"
        ><div
          v-if="rows[item.index]"
          class="comparison-row"
          role="row"
          :style="{
            gridTemplateColumns: template,
            transform: `translateY(${item.start}px)`,
            height: `${item.size}px`,
          }"
        >
          <div role="rowheader">
            <span>{{ rows[item.index]!.label }}</span
            ><code :title="rows[item.index]!.key">{{
              rows[item.index]!.key
            }}</code>
          </div>
          <div
            v-for="(value, index) in rows[item.index]!.values"
            :key="index"
            role="cell"
            :class="{
              different: !rows[item.index]!.same,
              missing: value === null,
            }"
            :title="value ?? t('Fehlender Wert', 'Missing value')"
          >
            {{ value ?? '∅' }}
          </div>
        </div></template
      >
    </div>
  </div>
</template>
