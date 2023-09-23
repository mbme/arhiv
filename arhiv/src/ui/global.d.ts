import { DataSchema } from 'utils/schema';

declare global {
  interface Window {
    BASE_PATH: string;
    SCHEMA: DataSchema;
  }
}
