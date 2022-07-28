import { Page } from 'puppeteer-core';

export async function runHelpers(page: Page): Promise<void> {
  await Promise.all([passSteamAgeCheck(page)]);
}

async function passSteamAgeCheck(page: Page): Promise<void> {
  if (!page.url().includes('store.steampowered.com/agecheck/')) {
    return;
  }

  await page.$eval('#ageYear', (yearSelect) => {
    (yearSelect as HTMLSelectElement).value = '1986';
  });

  await Promise.all([
    page.waitForNavigation({
      waitUntil: 'networkidle2',
    }),
    page.click('#view_product_page_btn'),
  ]);
}
