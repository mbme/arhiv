import { Page } from 'puppeteer-core';
import { Context } from './context';
import { getImageSrc, getListStr, getText, Obj, removeEl } from './utils';

async function passAgeCheck(page: Page): Promise<void> {
  if (!page.url().includes('store.steampowered.com/agecheck/')) {
    return;
  }

  await page.$eval('#ageYear', (yearSelect) => {
    (yearSelect as HTMLSelectElement).value = '1986';
  });

  await Promise.all([page.waitForNavigation(), page.click('#view_product_page_btn')]);
}

export async function extractGameFromSteam(url: string, context: Context): Promise<boolean> {
  // https://store.steampowered.com/app/814380/Sekiro_Shadows_Die_Twice__GOTY_Edition/
  if (!url.includes('store.steampowered.com/app/')) {
    return false;
  }

  const page = await context.newPage(url);
  await passAgeCheck(page);

  const data: Obj = {};

  const cover_src = await getImageSrc(page, '.game_header_image_full');
  data.cover = await context.channel.createAttachment(cover_src);

  data.name = await getText(page, '#appHubAppName');
  data.release_date = await getText(page, '.release_date .date');
  data.developers = await getListStr(page, '#developers_list a');

  await removeEl(page, '#game_area_description h2'); // remove "About this game"
  data.description = await getText(page, '#game_area_description');

  await context.channel.createDocument('game', '', data);

  return true;
}
