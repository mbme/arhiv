import { DataSchema } from 'utils/schema';
import { AppController } from 'controller';

type Features = {
  use_local_storage: boolean;
};

declare global {
  interface Window {
    BASE_PATH: string;
    SCHEMA: DataSchema;
    FEATURES: Features;
    MIN_PASSWORD_LENGTH: number;
    CREATE_ARHIV: boolean;

    APP: AppController;
  }
}

export {};
