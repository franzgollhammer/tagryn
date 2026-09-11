import { computed } from 'vue';
import { useWorkspace } from './stores/workspace';

export function useI18n() {
  const store = useWorkspace();
  const de = computed(() => store.preferences.language === 'de');
  const t = (german: string, english: string): string =>
    de.value ? german : english;
  return { t, de };
}
