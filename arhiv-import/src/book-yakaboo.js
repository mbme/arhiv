/* eslint-env node */

const LANGUAGE_TRANSLATIONS = {
  'Українська': 'Ukrainian',
  'Англійська': 'English',
  'Російська': 'Russian',
};

async function extractBookFromYakaboo(url, browser) {
  if (!url.includes('www.yakaboo.ua/ua/')) {
    return null;
  }

  const page = await browser.newPage();
  await page.goto(url);

  const data = {
    title: await page.$eval('#product-title h1', node => node.innerText),
  };

  await page.$eval("a[href='#tab-description']", node => node.click());
  data['description'] = await page.$eval('.description-shadow', node => node.innerText);

  await page.$eval("a[href='#tab-attributes']", node => node.click());

  for (const row of await page.$$('#product-attribute-specs-table tr')) {
    const [prop, value] = await row.$$eval('td', nodes => nodes.map(node => node.innerText));

    switch (prop) {
      case 'Автор': {
        data['authors'] = value;
        break;
      }

      case 'Мова': {
        data['language'] = LANGUAGE_TRANSLATIONS[value];
        break;
      }

      case 'Рік видання': {
        data['publication_date'] = value;
        break;
      }

      case 'Перекладач': {
        data['translators'] = value;
        break;
      }

      case 'Видавництво': {
        data['publisher'] = value;
        break;
      }

      case 'Кількість сторінок': {
        data['pages'] = value;
        break;
      }

      case 'ISBN': {
        data['ISBN'] = value;
        break;
      }

      default: break;
    }
  }

  return data;
}

module.exports = {
  extractBookFromYakaboo,
};
