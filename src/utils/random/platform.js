export const getRandomBytes = __BROWSER__
                ? bytes => window.crypto.getRandomValues(new Uint8Array(bytes))
                : require('crypto').randomBytes
