import { DataSchema } from 'utils/schema';

type Features = {
  scraper: boolean;
  use_local_storage: boolean;
};

declare global {
  interface Window {
    BASE_PATH: string;
    SCHEMA: DataSchema;
    FEATURES: Features;
  }
}
