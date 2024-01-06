/* eslint-disable @typescript-eslint/no-floating-promises */

import 'utils/test-env-setup';

import assert from 'node:assert';
import { describe, it } from 'node:test';
import { suspensify } from './suspense';

describe('suspensify()', () => {
  it('returns a value when promise resolved', async () => {
    const suspender = suspensify<number>(new Promise((resolve) => resolve(1)));

    try {
      // since promise.then() is always async, it will always throw on first execution
      suspender.read();

      assert.fail('must throw');
    } catch (promise) {
      await promise;
    }

    assert.equal(suspender.read(), 1);
  });

  it('throws an error when promise rejected', async () => {
    const suspender = suspensify<number>(
      new Promise((_resolve, reject) => reject(new Error('test'))),
    );

    try {
      // since promise.then() is always async, it will always throw on first execution
      suspender.read();

      assert.fail('must throw');
    } catch (promise) {
      await promise;
    }

    assert.throws(() => {
      suspender.read();
    }, /test/);
  });
});
