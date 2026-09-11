import { chromium } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
const out = resolve(import.meta.dirname, '../artifacts/screenshots');
await mkdir(out, { recursive: true });
const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: 1440, height: 960 },
  deviceScaleFactor: 1,
});
const errors = [];
page.on('pageerror', (error) => errors.push(error.message));
await page.goto('http://127.0.0.1:1420/?review=1');
await page.getByRole('listbox').getByRole('option').first().waitFor();
await page
  .getByRole('option', {
    name: 'Alpine Licht_001 – Österreich.jpg',
    exact: true,
  })
  .click();
await page.locator('.image-preview img').waitFor();
await page.screenshot({ path: join(out, 'workspace-light.png') });
await page.getByRole('button', { name: 'Einstellungen', exact: true }).click();
await page.getByRole('button', { name: 'Dunkel', exact: true }).click();
await page.getByRole('button', { name: 'Fertig', exact: true }).click();
await page.screenshot({ path: join(out, 'workspace-dark.png') });
await page
  .getByRole('button', { name: 'Thumbnail-Raster', exact: true })
  .click();
await page.locator('.thumbnail-image img').first().waitFor();
await page.screenshot({ path: join(out, 'grid-dark.png') });
await page.getByRole('button', { name: 'Listenansicht', exact: true }).click();
await page
  .getByRole('listbox')
  .getByRole('option')
  .nth(1)
  .click({ modifiers: ['Meta'] });
await page
  .getByRole('button', { name: 'Vergleichen', exact: true })
  .first()
  .click();
await page.getByRole('dialog').waitFor();
await page.screenshot({ path: join(out, 'comparison-dark.png') });
await page.getByRole('button', { name: 'Fertig', exact: true }).click();
await page.getByRole('listbox').getByRole('option').first().click();
await page
  .getByRole('button', { name: 'Bearbeiten', exact: true })
  .first()
  .click();
await page
  .getByRole('textbox', { name: 'Titel', exact: true })
  .fill('Ein neuer Titel');
await page
  .getByRole('button', { name: 'Änderungen vormerken', exact: true })
  .click();
await page.locator('.pending-tray').waitFor();
await page.screenshot({ path: join(out, 'pending-dark.png') });
await page.getByRole('tab', { name: 'Alle Metadaten', exact: true }).click();
await page
  .getByRole('button', { name: 'Technische Tags anzeigen', exact: true })
  .click();
await page.screenshot({ path: join(out, 'metadata-dark.png') });
await page.setViewportSize({ width: 1000, height: 720 });
await page.screenshot({ path: join(out, 'compact-dark.png') });
await page.getByRole('button', { name: 'Einstellungen', exact: true }).click();
await page.getByRole('button', { name: 'Hell', exact: true }).click();
await page
  .getByRole('combobox', { name: 'Sprache', exact: true })
  .selectOption('en');
await page.getByRole('button', { name: 'Done', exact: true }).click();
await page.screenshot({ path: join(out, 'compact-light-english.png') });
await page.keyboard.press('Meta+k');
await page
  .getByRole('textbox', { name: 'Search commands', exact: true })
  .fill('appearance');
await page.keyboard.press('Enter');
await page
  .getByRole('heading', { name: 'Appearance & language', exact: true })
  .waitFor();
await page.getByRole('button', { name: 'Done', exact: true }).click();
const overflow = await page.evaluate(() => ({
  body: document.body.scrollWidth > innerWidth,
  shell: document.querySelector('.app-shell').scrollWidth > innerWidth,
}));
await page.getByRole('button', { name: 'Settings', exact: true }).click();
await page.getByRole('slider').focus();
await page.keyboard.press('End');
await page.getByRole('button', { name: 'Done', exact: true }).click();
await page.setViewportSize({ width: 900, height: 620 });
const largeText = await page.evaluate(() => ({
  fontSize: getComputedStyle(document.documentElement).fontSize,
  overflow: document.body.scrollWidth > innerWidth,
  clippedTags: [...document.querySelectorAll('.tag-row')].filter(
    (row) =>
      row.querySelector('.tag-value')?.getBoundingClientRect().bottom >
      row.getBoundingClientRect().bottom + 1,
  ).length,
}));
await page.screenshot({ path: join(out, '900-light-17px.png') });
await writeFile(
  join(out, 'visual-check.json'),
  JSON.stringify(
    {
      pageErrors: errors,
      overflow,
      largeText,
      checked: [
        'light',
        'dark',
        '1000×720',
        '1440×960',
        'grid',
        'compare',
        'stage edits',
        'technical tags',
        'English',
        'command palette keyboard',
      ],
    },
    null,
    2,
  ),
);
await browser.close();
if (
  errors.length ||
  overflow.body ||
  largeText.overflow ||
  largeText.clippedTags > 0 ||
  largeText.fontSize !== '17px'
)
  throw new Error(JSON.stringify({ errors, overflow, largeText }));
console.log(JSON.stringify({ result: 'passed', artifacts: out, overflow }));
