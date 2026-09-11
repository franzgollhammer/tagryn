import { test, expect } from '@playwright/test';
test('scaled metadata rows remain visible after changing technical view', async ({
  page,
}) => {
  await page.goto('/?review=1');
  await page.getByRole('listbox').getByRole('option').first().click();
  await page.getByRole('tab', { name: 'Alle Metadaten', exact: true }).click();
  await expect(page.locator('.tag-row').first()).toBeVisible();
  await page
    .getByRole('button', { name: 'Technische Tags anzeigen', exact: true })
    .click();
  await page
    .getByRole('button', { name: 'Einstellungen', exact: true })
    .click();
  await page.getByRole('slider').focus();
  await page.keyboard.press('End');
  await page.getByRole('button', { name: 'Fertig', exact: true }).click();
  await page.setViewportSize({ width: 900, height: 620 });
  await expect
    .poll(() =>
      page.evaluate(() => ({
        fontSize: getComputedStyle(document.documentElement).fontSize,
        overflow: document.body.scrollWidth > innerWidth,
        rows: document.querySelectorAll('.tag-row').length > 0,
        clipped: [...document.querySelectorAll('.tag-row')].filter((row) => {
          const value = row.querySelector('.tag-value');
          return (
            value &&
            value.getBoundingClientRect().bottom >
              row.getBoundingClientRect().bottom + 1
          );
        }).length,
      })),
    )
    .toEqual({ fontSize: '17px', overflow: false, rows: true, clipped: 0 });
});
test('empty workspace is usable without a backend', async ({ page }) => {
  await page.goto('/');
  await expect(
    page.getByRole('heading', { name: 'Jedes Bild hat mehr zu erzählen.' }),
  ).toBeVisible();
  await page
    .getByRole('button', { name: 'Einstellungen', exact: true })
    .click();
  await page.getByRole('button', { name: 'Dunkel', exact: true }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.getByRole('button', { name: 'Fertig', exact: true }).click();
});
test('review data enables virtualized browsing and diffs without fake engine writes', async ({
  page,
}) => {
  await page.goto('/?review=1');
  await expect(
    page.getByRole('listbox').getByRole('option').first(),
  ).toBeVisible();
  await page
    .getByRole('listbox')
    .getByRole('option')
    .nth(1)
    .click({ modifiers: [process.platform === 'darwin' ? 'Meta' : 'Control'] });
  await page
    .getByRole('button', { name: 'Vergleichen', exact: true })
    .first()
    .click();
  await expect(
    page.getByRole('table', { name: 'Metadatenvergleich' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Fertig', exact: true }).click();
  await page.keyboard.press(
    process.platform === 'darwin' ? 'Meta+k' : 'Control+k',
  );
  await page
    .getByRole('textbox', { name: 'Befehl suchen' })
    .fill('Darstellung');
  await page.keyboard.press('Enter');
  await expect(
    page.getByRole('heading', { name: 'Darstellung & Sprache' }),
  ).toBeVisible();
});
