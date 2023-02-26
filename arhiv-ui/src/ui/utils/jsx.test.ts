import test from 'ava';
import { suspensify } from './jsx';

test('suspensify() returns a value when promise resolved', async (t) => {
  const suspender = suspensify<number>(new Promise((resolve) => resolve(1)));

  try {
    // since promise.then() is always async, it will always throw on first execution
    suspender.read();

    t.fail('must throw');
  } catch (promise) {
    await promise;
  }

  t.is(suspender.read(), 1);
});

test('suspensify() throws an error when promise rejected', async (t) => {
  const suspender = suspensify<number>(
    new Promise((_resolve, reject) => reject(new Error('test')))
  );

  try {
    // since promise.then() is always async, it will always throw on first execution
    suspender.read();

    t.fail('must throw');
  } catch (promise) {
    await promise;
  }

  t.throws(() => suspender.read(), { message: 'test' });
});
