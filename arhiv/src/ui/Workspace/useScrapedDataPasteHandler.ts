import { ScrapeResult, ScraperResultsContainer } from 'scraper';
import {
  BOOK_DOCUMENT_TYPE,
  DocumentData,
  DocumentType,
  FILM_DOCUMENT_TYPE,
  GAME_DOCUMENT_TYPE,
} from 'dto';
import { isDefined } from 'utils';
import { useClipboardPasteHandler } from 'utils/hooks';

const SCRAPE_RESULTS_TYPE = 'scrape-results-container';

function tryParseScraperResultsContainer(rawValue: string): ScrapeResult[] {
  if (!rawValue.includes(SCRAPE_RESULTS_TYPE)) {
    return [];
  }

  try {
    const value = JSON.parse(rawValue) as unknown;

    if (
      !value ||
      typeof value !== 'object' ||
      !('@type' in value) ||
      value['@type'] !== SCRAPE_RESULTS_TYPE
    ) {
      return [];
    }

    return (value as ScraperResultsContainer).results;
  } catch (_e) {
    /* empty */
  }

  return [];
}

export type ScrapedData = {
  documentType: DocumentType;
  data: DocumentData;
};

function scrapeResultToScrapedData(result: ScrapeResult): ScrapedData | undefined {
  console.log('Got scrape result:', result);
  switch (result.data?.typeName) {
    case 'YakabooBook': {
      const { data } = result;

      return {
        documentType: BOOK_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'title': data.title,
          'description': data.description,
          'authors': data.authors,
          'language': data.language,
          'publication_date': data.publicationDate,
          'translators': data.translators,
          'publisher': data.publisher,
          'pages': data.pages,
        },
      };
    }
    case 'IMDBFilm': {
      const { data } = result;

      return {
        documentType: FILM_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'title': data.title,
          'release_date': data.releaseDate,
          'original_language': data.originalLanguage,
          'countries_of_origin': data.countriesOfOrigin,
          'description': data.description,
          'duration': data.duration,
          'seasons': data.seasons ?? null,
          'episodes': data.episodes ?? null,
          'creators': data.creators,
          'cast': data.cast,
        },
      };
    }
    case 'MyAnimeListAnime': {
      const { data } = result;

      return {
        documentType: FILM_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'title': data.title,
          'release_date': data.releaseDate,
          'original_language': 'Japanese',
          'countries_of_origin': 'Japan',
          'creators': data.creators,
          'duration': data.duration,
          'description': data.description,
        },
      };
    }
    case 'SteamGame': {
      const { data } = result;

      return {
        documentType: GAME_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'name': data.name,
          'release_date': data.releaseDate,
          'developers': data.developers,
          'description': data.description,
        },
      };
    }
    default:
      console.warn('Unsupported scrape result: %s', result.scraperName);
      return undefined;
  }
}

export function useScrapedDataPasteHandler(handler: (data: ScrapedData[]) => void) {
  useClipboardPasteHandler(async (data) => {
    const textItems = await Promise.all(
      [...data.items]
        .filter((item) => item.kind === 'string')
        .map(
          (item) =>
            new Promise<string>((resolve) => {
              item.getAsString(resolve);
            }),
        ),
    );

    const scrapedData = textItems
      .flatMap(tryParseScraperResultsContainer)
      .map(scrapeResultToScrapedData)
      .filter(isDefined);

    handler(scrapedData);
  });
}
